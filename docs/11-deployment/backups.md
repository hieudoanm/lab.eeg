## 13. Backups and Recovery

Define a recovery policy for every durable data store.

### PostgreSQL

Document:

- Backup frequency.
- Retention.
- Recovery procedure.
- Who can restore.
- Recovery point objective.
- Recovery time objective.

Verify the actual backup capabilities of the selected service and configuration.

### Object storage

Document:

- Recording retention.
- Versioning or generation behaviour where enabled.
- Deletion policy.
- Recovery options.
- How database metadata maps to stored objects.

### Recovery testing

A backup is not proven useful until restoration has been tested.

For a portfolio project:

- Document the restore procedure.
- Perform a test restore in a disposable environment.
- Verify that representative metadata and synthetic recording references remain consistent.
- Record any limitations.

