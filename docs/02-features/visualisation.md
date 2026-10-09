## 7. Asynchronous Analysis Jobs

**Priority:** P0

### Description

Researchers submit descriptive analyses that execute in a background worker rather than blocking an API request.

### Initial analysis

Implement one well-tested descriptive analysis containing:

- Mean.
- Standard deviation.
- Minimum.
- Maximum.
- Peak-to-peak range.
- Valid sample count.

Results should be computed per channel.

The implementation must document the standard deviation convention and how invalid or non-finite values are treated.

### Submission workflow

1. The user opens a ready dataset.
2. The user selects the supported analysis.
3. The frontend submits an analysis request.
4. The API validates permissions, parameters, and dataset state.
5. The API creates a job record and schedules work.
6. The API returns the job resource.
7. A worker executes the computation.
8. The worker persists the result and final job state.
9. The frontend refreshes or polls the job status.
10. The user opens the result.

### Job states

| State       | Meaning                                                                |
| ----------- | ---------------------------------------------------------------------- |
| `queued`    | Accepted but not yet executing                                         |
| `running`   | A worker is processing the job                                         |
| `succeeded` | Computation and result persistence completed                           |
| `failed`    | Processing failed and will not be retried automatically                |
| `cancelled` | Processing was cancelled according to the supported cancellation rules |

State transitions must be validated and persisted.

### Result view

The result page should show:

- Analysis type.
- Job status.
- Dataset version.
- Algorithm version.
- Parameters.
- Submission and completion timestamps.
- Per-channel statistics.
- Safe failure information, if applicable.

### Reliability requirements

- Job submission must be idempotent under the documented conditions.
- Workers must tolerate duplicate delivery.
- Transient failures must use bounded retries.
- Permanent input errors must not be retried indefinitely.
- A worker crash must not leave jobs permanently ambiguous.
- Results must not be persisted as successful before computation and validation finish.
- Database state and queue publication must be coordinated to avoid silently losing accepted jobs.

A transactional outbox is a suitable approach if the chosen queue cannot participate in the database transaction.

### Edge cases

- Dataset is archived before the job starts.
- User loses access after submission.
- Worker crashes during computation.
- Queue temporarily becomes unavailable.
- Same request is submitted repeatedly.
- Result persistence fails after computation.
- Job exceeds its execution limit.
- Frontend closes while the worker continues.

