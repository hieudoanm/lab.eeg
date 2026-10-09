## 10. Phase 6 — Analysis Pipeline

### Objectives

Implement durable asynchronous analysis execution.

### Initial analysis

`descriptive_statistics`

For each channel, calculate:

- Mean.
- Standard deviation.
- Minimum.
- Maximum.
- Peak-to-peak range.
- Valid sample count.

Document the treatment of missing or non-finite samples. Do not silently produce misleading statistics.

### Tasks

- [x] Define analysis request and parameter schemas.
- [x] Implement analysis submission.
- [x] Implement idempotency.
- [x] Persist job records and outbox events transactionally.
- [x] Implement outbox dispatch.
- [x] Configure Redis and Celery.
- [x] Implement the analysis worker.
- [x] Load recordings from private object storage.
- [x] Validate worker input independently.
- [x] Compute statistics using NumPy.
- [x] Persist validated results and provenance.
- [x] Implement job status polling.
- [x] Implement retry handling.
- [x] Implement stale-job recovery.
- [x] Implement cooperative cancellation if feasible.
- [x] Add worker integration tests.

### Required job states

- `queued`
- `running`
- `succeeded`
- `failed`
- `cancelled`

### Reliability acceptance criteria

- Submitting an analysis returns promptly.
