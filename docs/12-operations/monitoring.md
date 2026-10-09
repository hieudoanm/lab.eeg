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

