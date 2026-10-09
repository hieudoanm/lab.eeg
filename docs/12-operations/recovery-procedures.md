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

