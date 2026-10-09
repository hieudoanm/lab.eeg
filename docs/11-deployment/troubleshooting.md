## 18. Troubleshooting Guide

### API cannot connect to PostgreSQL

Check:

- Database availability.
- Connection-string format.
- Network configuration.
- Authentication and permissions.
- Connection-pool limits.
- Secret configuration.
- Database connection exhaustion.

Do not print passwords or full connection strings into logs.

### Worker does not process jobs

Check:

- Broker availability.
- Broker authentication and TLS.
- Worker deployment status.
- Queue configuration.
- Task registration.
- Dispatch failures in the outbox.
- Queue depth.
- Worker concurrency and resource limits.

Inspect job and outbox records before manually resubmitting work.

### Dataset upload remains pending

Check:

- Upload-session state.
- Signed URL expiry.
- Object existence.
- Object size and checksum.
- Validation-worker health.
- Validation error records.
- Object-storage permissions.

Do not manually mark the dataset ready without verifying the recording.

### Analysis remains queued

Check:

- Outbox dispatch status.
- Broker connectivity.
- Worker availability.
- Retry policy.
- Resource limits.
- Job state and lease information.

Use the documented recovery procedure rather than directly modifying production database rows without an approved process.

### Deployment succeeds but the application fails

Check:

- Environment variables.
- Health-check configuration.
- API base URL.
- Database migration status.
- Service identity permissions.
- Frontend/backend contract compatibility.
- Worker and broker connectivity.

