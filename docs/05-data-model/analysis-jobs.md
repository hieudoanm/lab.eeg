## 12. `analysis_jobs`

Represents a requested scientific computation.

Suggested columns:

| Column              | Type                  | Constraints / purpose         |
| ------------------- | --------------------- | ----------------------------- |
| `id`                | UUID                  | Primary key                   |
| `organisation_id`   | UUID                  | Tenant scope                  |
| `dataset_id`        | UUID                  | Source dataset                |
| `requested_by`      | UUID                  | Requesting user               |
| `analysis_type`     | TEXT                  | Supported analysis identifier |
| `algorithm_version` | TEXT                  | Algorithm version             |
| `parameters`        | JSONB                 | Validated analysis parameters |
| `status`            | TEXT                  | Job lifecycle                 |
| `attempt_count`     | INTEGER               | Number of execution attempts  |
| `max_attempts`      | INTEGER               | Retry limit                   |
| `error_code`        | TEXT, nullable        | Safe error classification     |
| `error_message`     | TEXT, nullable        | Sanitised diagnostic message  |
| `idempotency_key`   | TEXT, nullable        | Request deduplication key     |
| `requested_at`      | TIMESTAMPTZ           | Submission time               |
| `started_at`        | TIMESTAMPTZ, nullable | Execution start               |
| `completed_at`      | TIMESTAMPTZ, nullable | Terminal time                 |
| `lease_expires_at`  | TIMESTAMPTZ, nullable | Optional worker lease         |
| `created_at`        | TIMESTAMPTZ           | Record creation               |
| `updated_at`        | TIMESTAMPTZ           | Last update                   |

Suggested states:

- `queued`
- `running`
- `succeeded`
- `failed`
- `cancelled`

Constraints:

- Dataset and job must belong to the same organisation.
- Attempt counts must be non-negative.
- `max_attempts` must be positive.
- Analysis type must be supported.
- Parameters must pass schema validation before submission.
- Terminal states must not transition back to `running` without creating a new execution according to the documented retry model.
- `completed_at` must be populated when a terminal state is reached.
- The dataset version and analysis configuration used by a successful job must remain identifiable.

### Idempotency design

A job's idempotency scope should be explicit. A suitable initial scope is the authenticated user and organisation, combined with an endpoint or operation identifier and an idempotency key.

Do not assume that the same parameters always mean the same user intent. Two intentionally separate analysis runs may use identical parameters.

### Indexes

