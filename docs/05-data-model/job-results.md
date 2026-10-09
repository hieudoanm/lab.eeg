
- `(organisation_id, requested_at DESC)`
- `(organisation_id, status, requested_at DESC)`
- `(organisation_id, dataset_id, requested_at DESC)`
- Index on `lease_expires_at` for stale-job recovery if leases are implemented.

## 13. `analysis_results`

Stores the output of a completed analysis.

Suggested columns:

| Column                    | Type           | Constraints / purpose       |
| ------------------------- | -------------- | --------------------------- |
| `id`                      | UUID           | Primary key                 |
| `organisation_id`         | UUID           | Tenant scope                |
| `job_id`                  | UUID           | Source analysis job         |
| `dataset_id`              | UUID           | Source dataset              |
| `dataset_checksum_sha256` | TEXT, nullable | Input content identifier    |
| `algorithm_version`       | TEXT           | Computation version         |
| `result_schema_version`   | TEXT           | Result structure version    |
| `results`                 | JSONB          | Validated numerical output  |
| `provenance`              | JSONB          | Relevant execution metadata |
| `created_at`              | TIMESTAMPTZ    | Persistence time            |

Constraints:

- Each job produces at most one canonical result for the initial analysis model.
- `job_id` is unique.
- Result, job, dataset, and organisation relationships must be consistent.
- Results may only be committed as successful output after validation.
- Result schema and algorithm versions are required.

If a later analysis produces multiple artefacts, use a dedicated result-artefact table rather than overloading one JSON field with unrelated storage concerns.

### Result representation

For the initial descriptive analysis, a JSON structure may contain one entry per channel with:

- Channel index.
- Channel name.
- Mean.
- Standard deviation.
- Minimum.
- Maximum.
- Peak-to-peak range.
- Valid sample count.
- Unit where known.

Validate the result structure before persistence.

### Indexes

- Unique `(job_id)`.
- `(organisation_id, dataset_id, created_at DESC)`.

## 14. `outbox_events`

Supports reliable publication of background work.

Suggested columns:

| Column            | Type                  | Constraints / purpose              |
| ----------------- | --------------------- | ---------------------------------- |
| `id`              | UUID                  | Primary key                        |
| `organisation_id` | UUID, nullable        | Tenant context where applicable    |
| `event_type`      | TEXT                  | Event name                         |
| `aggregate_type`  | TEXT                  | Related resource type              |
| `aggregate_id`    | UUID                  | Related resource identifier        |
| `payload`         | JSONB                 | Minimal validated dispatch payload |
| `created_at`      | TIMESTAMPTZ           | Creation time                      |
| `published_at`    | TIMESTAMPTZ, nullable | Publication time                   |
| `attempt_count`   | INTEGER               | Dispatch attempts                  |
| `next_attempt_at` | TIMESTAMPTZ, nullable | Scheduled retry                    |
| `last_error_code` | TEXT, nullable        | Safe failure classification        |

Constraints:

- Payload must not contain secrets or raw scientific arrays.
- Published events must not be republished indefinitely.
- Dispatch retries must be bounded or moved to an operationally visible failure state.
- Duplicate delivery must be safe.

A dispatcher may use row locking with `FOR UPDATE SKIP LOCKED` to claim events without multiple dispatchers continuously processing the same rows.

## 15. `idempotency_keys`

Provides a durable record of state-changing API requests that require deduplication.

Suggested columns:

| Column            | Type              | Constraints / purpose              |
| ----------------- | ----------------- | ---------------------------------- |
| `id`              | UUID              | Primary key                        |
| `organisation_id` | UUID              | Tenant scope                       |
| `user_id`         | UUID              | Requesting user                    |
| `operation`       | TEXT              | API operation identifier           |
| `key`             | TEXT              | Client-provided idempotency key    |
| `request_hash`    | TEXT              | Hash of canonicalised request data |
| `response_status` | INTEGER, nullable | Recorded response status           |
| `response_body`   | JSONB, nullable   | Safe response representation       |
| `resource_id`     | UUID, nullable    | Created resource identifier        |
| `created_at`      | TIMESTAMPTZ       | First request time                 |
| `expires_at`      | TIMESTAMPTZ       | Retention deadline                 |

Constraints:

- Unique `(organisation_id, user_id, operation, key)`.
- Reuse with a different request hash must be rejected.
- Concurrent requests using the same key must not create duplicate logical operations.
- Sensitive response fields must not be stored.
- Retention and cleanup behaviour must be documented.

Use idempotency records for operations where duplicate execution would be harmful or confusing, especially analysis submission and upload finalisation.

## 16. `audit_events`

Stores an append-oriented history of important actions.

Suggested columns:

| Column            | Type           | Constraints / purpose               |
| ----------------- | -------------- | ----------------------------------- |
| `id`              | UUID           | Primary key                         |
| `organisation_id` | UUID, nullable | Organisation context                |
| `actor_user_id`   | UUID, nullable | Acting user                         |
| `action`          | TEXT           | Event action                        |
| `resource_type`   | TEXT           | Affected resource type              |
| `resource_id`     | UUID, nullable | Affected resource                   |
| `request_id`      | TEXT, nullable | Correlation identifier              |
| `metadata`        | JSONB          | Minimal non-sensitive event details |
| `created_at`      | TIMESTAMPTZ    | Event time                          |

Example actions:

- `study.created`
- `study.updated`
- `study.archived`
- `dataset.uploaded`
- `dataset.validation_failed`
- `analysis.submitted`
- `analysis.completed`
- `analysis.failed`
- `membership.role_changed`
- `membership.removed`

Rules:

- Audit records should be append-only through normal application workflows.
- Public application endpoints must not allow arbitrary modification of audit history.
- Record only necessary metadata.
- Do not store tokens, passwords, signed URLs, or raw signal data.
- Where an operation requires an audit event for integrity, persist the event in the same transaction as the related database change when possible.

## 17. Cross-Tenant Referential Integrity

Tenant isolation should be reflected in the relational schema.

For tenant-owned entities, prefer composite foreign keys where useful.

Example:

```sql
CREATE UNIQUE INDEX uq_studies_organisation_id_id
    ON studies (organisation_id, id);

ALTER TABLE datasets
    ADD CONSTRAINT fk_datasets_study_same_organisation
    FOREIGN KEY (organisation_id, study_id)
    REFERENCES studies (organisation_id, id);
```

The exact migration order must account for existing constraints and table creation order.

Apply the same principle to analysis jobs and datasets where both records carry an organisation identifier.

This is defence in depth. It does not replace permission checks.

