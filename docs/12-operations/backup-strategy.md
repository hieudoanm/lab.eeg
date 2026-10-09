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

