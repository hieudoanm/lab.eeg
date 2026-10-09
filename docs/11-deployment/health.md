## 11. Health Checks and Readiness

Implement separate health endpoints.

### Liveness

`GET /health/live`

Indicates that the API process is alive and able to respond.

This endpoint should not fail solely because an external dependency is temporarily unavailable.

### Readiness

`GET /health/ready`

Indicates whether the API can accept work under its defined readiness policy.

Check required dependencies appropriately, such as database connectivity. Avoid expensive or unbounded checks.

Do not expose credentials, internal network details, or detailed dependency errors through public health responses.

### Worker health

Track:

- Worker process availability.
- Last successful task.
- Queue depth where available.
- Job age.
- Failed task count.
- Retry count.
- Stale running jobs.

A healthy HTTP endpoint does not prove that the asynchronous processing pipeline is healthy.

