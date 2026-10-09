
## 11. Background Worker Tests

Background jobs are distributed operations. Assume messages may be delayed, delivered more than once, or processed after a worker restart.

### Job lifecycle

Test:

- `queued` transitions to `running`.
- Successful execution transitions to `succeeded`.
- Expected execution errors transition to `failed` or trigger an explicitly permitted retry.
- Queued jobs can be cancelled.
- Running jobs follow the documented cancellation policy.
- Terminal jobs cannot be silently reopened.
- A worker checks persisted state before committing results.

### Idempotent execution

Deliver the same task more than once.

Verify that:

- Only one canonical result is committed.
- Repeated execution does not corrupt the dataset.
- A duplicate message cannot overwrite a successful result with a failure.
- Duplicate dispatch is safe.
- State transitions remain valid.

### Failure and recovery

Simulate:

- Worker termination during execution.
- Object storage temporarily unavailable.
- Database connection loss.
- Invalid analysis input.
- Outbox dispatch failure.
- A job that exceeds its lease or execution timeout.
- Exhausted retry attempts.

Verify that failures are visible and that retry behaviour is bounded.

### Avoid false confidence from eager mode

Celery eager mode can be useful for some unit tests, but it does not reproduce the behaviour of a real broker and worker process.

Use a real test broker or a representative integration environment for critical delivery, retry, and worker-lifecycle tests.

