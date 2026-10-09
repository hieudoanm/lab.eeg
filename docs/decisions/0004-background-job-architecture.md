# Decision Record 0004 — Background Job Architecture (Outbox + Celery)

**Date:** 2026-10-10  
**Status:** Accepted  
**Context:** Phase 6 — Analysis Pipeline

## Context

The analysis pipeline must run asynchronously because EEG recordings can be large and computation may take seconds to minutes. The API should accept a submission, persist a durable job record, and return promptly while a background worker executes the analysis.

Key requirements:

- The API and worker are separate processes (or containers) that do not share memory.
- Job state must survive API restarts and worker crashes.
- Duplicate deliveries (Celery retries, dispatcher retries, network partitions) must not corrupt results.
- Retries must be bounded and observable.
- Provenance (dataset identity/checksum, algorithm version, parameters, result schema version) must be recorded with every result.
- Cooperative cancellation should be supported for queued jobs; running jobs cannot be reliably interrupted.

## Decision

We implement a **transactional outbox pattern** backed by PostgreSQL, with **Celery + Redis** as the message broker.

### Architecture

```
┌──────────────┐     1. transaction     ┌──────────────────────────┐
│   API        │───────────────────────▶│ analysis_jobs (queued)   │
│  (submit)    │   job + outbox event   │ outbox_events (pending)  │
└──────────────┘                        │ idempotency_keys         │
                                        └──────────────────────────┘
                                                 │
                                                 │ 2. dispatcher (FOR UPDATE SKIP LOCKED)
                                                 ▼
                                        ┌──────────────────────────┐
                                        │      Redis (Celery)      │
                                        │   task: eeg-lab.run_analysis │
                                        └──────────────────────────┘
                                                 │
                                                 │ 3. worker claims job
                                                 ▼
                                        ┌──────────────────────────┐
                                        │      Worker              │
                                        │  - claim_job (running)   │
                                        │  - load/validate .npy    │
                                        │  - compute statistics    │
                                        │  - create_result +       │
                                        │    complete_job (atomic) │
                                        └──────────────────────────┘
```

### Key Design Choices

#### 1. Transactional Outbox

- When the API accepts a submission, it inserts the job (`queued`), an outbox event (`analysis.requested`), and an idempotency record **in one database transaction**.
- A committed job **always** has a pending outbox event; a rolled-back submission leaves neither.
- This eliminates the "job without dispatch" and "dispatch without job" failure modes.

#### 2. Dispatcher with Row-Level Locking

- A separate dispatcher process (or cron) claims pending events with `FOR UPDATE SKIP LOCKED`, publishes each to Celery via `send_task`, and marks the event published **in the same transaction**.
- The row lock is held across the publish call so two dispatchers can never publish the same event. This is acceptable because a Redis `send_task` is a fast, non-storage operation (sub-millisecond latency on local networks).

#### 3. Bounded Dispatch Retries with Exponential Backoff

- On publish failure, the dispatcher increments `attempt_count` and schedules the next attempt at `retry_delay_seconds * 2**attempt_count`.
- After `max_attempts` (default 10), the event is **parked** with `next_attempt_at = now + 100 years` and `last_error_code = "outbox_dispatch_exhausted"` so it stops retrying but remains visible for manual intervention.
- Unknown event types are parked immediately with `last_error_code = "outbox_unknown_event"`.

#### 4. Worker Retries Driven by Job State, Not Celery

- The worker claims the job atomically (`queued` → `running`, bump `attempt_count`, set `lease_expires_at`).
- **Celery does not retry** (`autoretry_for=()`, `retry_backoff=False`). All retries are driven by the durable job:
  - Transient failure (storage hiccup, database timeout) → `_handle_failure` requeues the job (`running` → `queued`) + emits a fresh outbox event.
  - Deterministic failure (checksum mismatch, invalid recording, bad parameters) → job fails immediately with a safe error code.
- This avoids Celery's in-memory retry state (lost on worker restart) and keeps retry logic visible in the database.

#### 5. Lease Exceeds Task Time Limit

- Lease duration = `max(job_lease_seconds, celery_task_time_limit_seconds + 30)`.
- The 30-second buffer ensures a worker that is still computing is never reclaimed by stale-job recovery.

#### 6. Stale-Job Recovery

- A periodic recovery task claims running jobs whose `lease_expires_at <= now`.
- If `attempt_count < max_attempts` → requeue + new outbox event (redelivered).
- If `attempt_count >= max_attempts` → fail with `error_code = "analysis_stale"`.

#### 7. Result Uniqueness via Unique Constraint

- `analysis_results` has a unique constraint on `job_id` (`uq_analysis_results_job_id`).
- `create_result` uses `ON CONFLICT DO NOTHING`. A duplicate worker delivery finds the existing row, reuses it, and returns `status = "skipped"`.

#### 8. Idempotency Key Scope

- Key = `(organisation_id, user_id, operation="analysis.submit", key_value)`.
- A repeated key with an identical request hash replays the original job (202, same job ID).
- A repeated key with a different request hash → 409 `idempotency_key_reuse`.
- TTL = `idempotency_ttl_seconds` (default 86400).

#### 9. Cancellation

- Only queued jobs can be cancelled (`queued` → `cancelled`).
- Already cancelled → return current job (idempotent 200).
- Running → 409 `analysis_running`.
- Succeeded/failed → 409 `analysis_not_cancellable`.
- No `cancel_requested_at` column in schema → no cooperative interruption of running workers (deliberate, Phase 8 may add it).

#### 10. Error Codes

| Code                          | Source     | Retryable | Meaning                                         |
| ----------------------------- | ---------- | --------- | ----------------------------------------------- |
| `dataset_not_ready`           | API        | No        | Submit rejected; dataset not validated          |
| `idempotency_key_reuse`       | API        | No        | Same key, different request body                |
| `results_not_ready`           | API        | —         | GET results while job not terminal              |
| `result_not_available`        | API        | —         | GET results on terminal non-success job         |
| `analysis_running`            | API        | —         | Cancel requested on running job                 |
| `analysis_not_cancellable`    | API        | —         | Cancel on succeeded/failed job                  |
| `analysis_input_unavailable`  | Worker     | Yes       | Object missing, size/checksum transient failure |
| `analysis_input_changed`      | Worker     | No        | Checksum/size mismatch vs validated dataset     |
| `analysis_input_invalid`      | Worker     | No        | Invalid .npy or manifest mismatch               |
| `analysis_type_unsupported`   | Worker     | No        | Only `descriptive_statistics` exists            |
| `analysis_parameters_invalid` | Worker     | No        | Parameters failed validation                    |
| `analysis_execution_failed`   | Worker     | Yes       | Catch-all for storage/DB hiccups                |
| `analysis_stale`              | Recovery   | No        | Job exceeded max attempts after lease expiry    |
| `outbox_dispatch_failed`      | Dispatcher | —         | Intermediate publish failure                    |
| `outbox_dispatch_exhausted`   | Dispatcher | —         | Event parked after max attempts                 |
| `outbox_unknown_event`        | Dispatcher | —         | Event type not in registry                      |

#### 11. Audit Actions

- `analysis.submitted`
- `analysis.completed`
- `analysis.failed`
- `analysis.cancelled`

#### 12. Result Payload & Provenance

- `results.channels[]` with optional fields (`mean`, `std`, `min`, `max`, `ptp`, `valid_sample_count`) — disabled metrics omitted.
- `provenance` includes `input_checksum_sha256`, `sampling_frequency_hz`, `channel_count`, `sample_count`, `parameters`.
- `result_schema_version = "1.0.0"` stamped on every row.

## Alternatives Considered

| Alternative                             | Rejected Because                                                                        |
| --------------------------------------- | --------------------------------------------------------------------------------------- |
| Celery native retries (`autoretry_for`) | In-memory state lost on worker restart; no visibility into retry count from API         |
| Kafka / RabbitMQ as primary queue       | Operational overhead; Redis is already in stack for Celery broker                       |
| Synchronous execution in API            | Blocks request thread; violates `NFR-05` (sub-second API response for submission)       |
| `cancel_requested_at` + worker polling  | Adds schema complexity; initial scope only requires cooperative queued-job cancellation |
| Result in Celery backend (Redis)        | Redis is ephemeral; PostgreSQL is the authoritative store for job state & results       |

## Consequences

**Positive:**

- All job state and retry logic is visible in PostgreSQL — debuggable and auditable.
- Duplicate deliveries at any layer (API idempotency, dispatcher retry, Celery redelivery, worker retry) are safe.
- No message loss: a committed job always has an outbox event; dispatcher retries until published or parked.
- Lease-based stale recovery handles worker crashes without manual intervention.

**Negative / Operational Burden:**

- Dispatcher must be running (or cron'd) for jobs to be delivered. If it stops, jobs pile up in `queued` with pending outbox events.
- Two processes (API + dispatcher + worker) must be deployed and monitored.
- Redis is a single point of failure for dispatch; however, it only holds transient tasks — PostgreSQL is the source of truth.

## Validation

- Unit tests: `tests/unit/test_statistics.py` (deterministic compute).
- API tests: `tests/api/test_analyses.py` (submission, idempotency, status, results, cancellation, tenancy).
- Integration tests: `tests/integration/test_analysis_execution.py` (execution lifecycle, retry, stale recovery, outbox dispatch with publish/retry/exhaustion, transactional rollback).
- All tests pass against real PostgreSQL and Redis.

## Related Documents

- `docs/06-data-model.md` §13–14 (analysis_jobs, analysis_results, outbox_events, idempotency_keys)
- `docs/07-api-spec.md` §11 (submit, idempotency, status, results, cancel)
- `docs/11-implementation-plan.md` §10 (Phase 6 tasks)
- `docs/10-testing-strategy.md` §11 (worker tests)
- `app/services/outbox.py`, `app/services/analysis_execution.py`, `app/workers/analysis_tasks.py`, `app/workers/outbox_dispatcher.py`

---

_This decision is recorded per the project's delivery rules (AGENTS.md §8.4)._
