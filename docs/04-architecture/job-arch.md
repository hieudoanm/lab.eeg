## 10. Analysis Job Architecture

### 10.1 Job submission

The API validates the request and creates a durable job record.

The record includes:

- Job ID.
- Organisation ID.
- Dataset ID and version.
- Analysis type.
- Validated parameters.
- Requested-by user ID.
- Idempotency information where applicable.
- Current state.
- Timestamps.

The API returns the job resource without waiting for the analysis to finish.

### 10.2 Dispatch reliability

A naive sequence of committing a job to PostgreSQL and then publishing a Redis message has a failure window: the database transaction may succeed while queue publication fails.

Use a transactional outbox or another explicitly documented reliable-dispatch mechanism.

With an outbox:

1. Create the job and outbox event in the same database transaction.
2. Commit both.
3. A dispatcher publishes pending outbox events.
4. The dispatcher marks an event as published after successful delivery.
5. Repeated publication remains safe because worker processing is idempotent.

### 10.3 Worker execution

The worker:

1. Receives a job identifier.
2. Loads the job from PostgreSQL.
3. Checks that its state permits execution.
4. Claims the job using a concurrency-safe transition.
5. Loads the relevant dataset and metadata.
6. Executes the validated analysis.
7. Validates the output.
8. Persists the result.
9. Marks the job successful in the appropriate transaction.
10. Emits structured operational logs.

If a failure occurs, the worker records an appropriate state and classifies whether retry is safe.

### 10.4 Concurrency

Do not allow multiple workers to commit competing results for the same logical job.

Use a conditional state update, row locking, a lease, or another explicitly tested concurrency mechanism.

A worker that discovers the job has already completed should exit without changing its result.

### 10.5 Retries

Use bounded retries with exponential backoff and jitter for transient failures.

Do not automatically retry invalid input, unsupported file formats, or deterministic scientific computation errors without a clear reason.

Define what happens when the retry limit is reached.

### 10.6 Stale jobs

Implement a recovery strategy for jobs left in `running` after a worker crash.

A lease or heartbeat can distinguish active processing from stale ownership.

Recovery must avoid restarting work that is still executing elsewhere.

