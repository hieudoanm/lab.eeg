## 7. Background Processing

### 7.1 Celery

Use Celery for asynchronous analysis tasks.

Responsibilities:

- Dispatch analysis jobs.
- Execute descriptive statistics.
- Record progress where meaningful.
- Handle transient failures.
- Retry eligible tasks with bounded backoff.
- Persist results.
- Record final job states.

Tasks should be small enough to test independently and should not rely on in-memory API state.

### 7.2 Redis

Use Redis as the Celery broker in the initial implementation.

Configure:

- Connection timeouts.
- Authentication and network restrictions in deployed environments.
- Task time limits.
- Retry policies.
- Queue monitoring.
- Appropriate persistence or managed-service durability settings.

Redis is not the source of truth for application job records. PostgreSQL remains authoritative for job state and results.

### 7.3 Idempotency and recovery

Design tasks to tolerate repeated delivery.

The implementation should account for:

- Duplicate task messages.
- Worker restarts.
- Transient database failures.
- Result persistence failures.
- Jobs that exceed their time limits.
- Tasks that finish after their lease or ownership period expires.

Use explicit job-state transitions and concurrency controls to prevent conflicting workers from corrupting the same logical job.

