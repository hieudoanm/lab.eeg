# Database Backup and Recovery Guidance

This document outlines backup and recovery procedures for the EEG Lab PostgreSQL database.

## Backup Strategy

### Automated Backups (Cloud SQL / Managed PostgreSQL)

For production deployments using managed PostgreSQL (e.g., Google Cloud SQL, AWS RDS, Azure Database):

1. **Enable automated daily backups** with point-in-time recovery (PITR)
2. **Retention period**: Minimum 7 days, recommended 30 days
3. **Backup window**: During low-traffic hours (e.g., 03:00 UTC)
4. **Cross-region replication**: Enable for disaster recovery

### Manual Backups (Self-hosted PostgreSQL)

For self-hosted deployments, use `pg_dump` with a cron job:

```bash
#!/bin/bash
# backup.sh - Run daily via cron

BACKUP_DIR="/var/backups/eeg-lab"
DATE=$(date +%Y%m%d_%H%M%S)
DB_NAME="eeg-lab"
DB_USER="eeg-lab"

mkdir -p "$BACKUP_DIR"

# Full dump with schema and data
pg_dump -U "$DB_USER" -d "$DB_NAME" \
  --format=custom \
  --compress=9 \
  --no-owner \
  --no-privileges \
  --file="${BACKUP_DIR}/eeg-lab_${DATE}.dump"

# Also create a plain SQL dump for inspection
pg_dump -U "$DB_USER" -d "$DB_NAME" \
  --format=plain \
  --no-owner \
  --no-privileges \
  --file="${BACKUP_DIR}/eeg-lab_${DATE}.sql"

# Retain last 30 days
find "$BACKUP_DIR" -name "eeg-lab_*.dump" -mtime +30 -delete
find "$BACKUP_DIR" -name "eeg-lab_*.sql" -mtime +30 -delete
```

### Backup Verification

Test restore procedures quarterly:

```bash
# Create a test database
createdb -U eeg-lab eeg-lab_test_restore

# Restore from dump
pg_restore -U eeg-lab -d eeg-lab_test_restore \
  --no-owner --no-privileges \
  /var/backups/eeg-lab/eeg-lab_20260115_030000.dump

# Verify row counts
psql -U eeg-lab -d eeg-lab_test_restore -c "
  SELECT 'users', COUNT(*) FROM users
  UNION ALL SELECT 'organisations', COUNT(*) FROM organisations
  UNION ALL SELECT 'studies', COUNT(*) FROM studies
  UNION ALL SELECT 'datasets', COUNT(*) FROM datasets
  UNION ALL SELECT 'analysis_jobs', COUNT(*) FROM analysis_jobs
  UNION ALL SELECT 'analysis_results', COUNT(*) FROM analysis_results;
"

# Clean up
dropdb -U eeg-lab eeg-lab_test_restore
```

## Recovery Procedures

### Point-in-Time Recovery (PITR)

For managed PostgreSQL with PITR enabled:

1. Identify the target timestamp (before data loss/corruption)
2. Initiate restore to a new instance at that timestamp
3. Verify data integrity
4. Swap DNS / connection strings to new instance

### Full Restore from Dump

```bash
# 1. Stop the API and worker processes
systemctl stop eeg-lab-api eeg-lab-worker eeg-lab-dispatcher

# 2. Drop and recreate the database
psql -U postgres -c "DROP DATABASE IF EXISTS eeg-lab;"
psql -U postgres -c "CREATE DATABASE eeg-lab OWNER eeg-lab;"

# 3. Restore from the latest good backup
pg_restore -U eeg-lab -d eeg-lab \
  --no-owner --no-privileges \
  --jobs=4 \
  /var/backups/eeg-lab/eeg-lab_20260115_030000.dump

# 4. Run any pending migrations (if schema changed since backup)
cd packages/backend && uv run python -m app.db.migrations upgrade

# 5. Restart services
systemctl start eeg-lab-api eeg-lab-worker eeg-lab-dispatcher
```

### Partial Recovery (Specific Tables)

If only specific tables are corrupted:

```bash
# Dump only the affected table from backup
pg_restore -U eeg-lab -d eeg-lab \
  --table=analysis_jobs \
  --data-only \
  --no-owner --no-privileges \
  /var/backups/eeg-lab/eeg-lab_20260115_030000.dump
```

## Object Storage Backup

Recordings are stored in object storage (local filesystem or GCS). Backups should include:

### Local Filesystem Adapter

```bash
# Sync to backup location
rsync -av --delete /var/lib/eeg-lab/uploads/ /backup/eeg-lab/uploads/
```

### Google Cloud Storage

Enable **Object Versioning** on the bucket:

```bash
gsutil versioning set on gs://eeg-lab-uploads
```

Set a **lifecycle rule** to delete old versions after 90 days.

## Recovery Time Objectives (RTO) / Recovery Point Objectives (RPO)

| Component                            | RTO       | RPO       | Notes                     |
| ------------------------------------ | --------- | --------- | ------------------------- |
| PostgreSQL (managed, PITR)           | < 30 min  | < 5 min   | Automated failover        |
| PostgreSQL (self-hosted, daily dump) | < 2 hours | 24 hours  | Manual restore            |
| Object Storage (GCS, versioned)      | < 15 min  | Near-zero | Versioning                |
| Object Storage (local, rsync)        | < 1 hour  | 24 hours  | Depends on sync frequency |

## Monitoring Backup Health

Alert on:

- Backup job failures (cron exit code != 0)
- Backup size significantly smaller than previous (> 50% drop)
- Backup age > 36 hours (for daily backups)
- PITR replication lag > 5 minutes

Example Prometheus alert:

```yaml
groups:
  - name: backup
    rules:
      - alert: BackupStale
        expr: time() - eeg-lab_backup_timestamp_seconds > 86400
        for: 1h
        labels:
          severity: critical
        annotations:
          summary: 'EEG Lab database backup is stale'
          description: 'Last successful backup was {{ $value | humanizeDuration }} ago'
```

## Testing Schedule

| Test                                   | Frequency     |
| -------------------------------------- | ------------- |
| Full restore to test environment       | Quarterly     |
| Partial table restore                  | Monthly       |
| PITR restore drill                     | Semi-annually |
| Object storage versioning verification | Monthly       |

## Contacts

- Primary: Platform Engineering team
- Secondary: Database Administrator
- Escalation: Engineering Lead

## Related Documentation

- `docs/12-deployment.md` — Deployment procedures
- `docs/06-data-model.md` — Schema reference
- `docs/decisions/0004-background-job-architecture.md` — Job state recovery
