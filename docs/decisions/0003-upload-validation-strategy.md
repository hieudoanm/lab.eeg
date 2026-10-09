# 0003 — Upload validation strategy

- Status: Accepted
- Date: 2026-10-10
- Phase: 5 (EEG Upload and Validation)

## Context

`docs/07-api-spec.md` originally described upload finalisation as an asynchronous
operation (`202 Accepted`, dataset status `validating`) performed by a worker.
The Phase 5 acceptance criteria require that a researcher can upload a synthetic
recording, that a dataset is not `ready` before validation, and that invalid
files are rejected with safe error messages.

The durable worker infrastructure is scheduled for Phase 6. Introducing the
worker and a job table/queue in Phase 5 would expand Phase 5 well beyond secure
ingestion and couple it to unfinished infrastructure. At the same time,
`docs/04-features.md` §5 and `docs/10-testing-strategy.md` describe an end-to-end
"upload → validation → ready" flow that must be demonstrable now.

The upload transport was also unspecified for local development. Object storage
in the MVP is a local filesystem adapter; there is no signed-URL provider
available in development.

## Decision

- **Validation runs synchronously during finalisation.** `POST .../finalise`
  verifies object existence, declared size, and (when supplied) the expected
  checksum, then validates the `.npy` header and returns `200 OK` with the
  terminal dataset state. A validation failure returns `422` (`invalid_recording`),
  transitions the dataset to `rejected` and the session to `failed`, and records
  a `dataset.validation_failed` audit event. `docs/07-api-spec.md` §10 reflects this
  behaviour; moving to `202 Accepted` and a `validating` state is reserved for
  Phase 6.
- **Uploads are backend-mediated in the MVP.** Creating a session returns
  `upload_method: "backend"` and a relative `upload_url` pointing at the
  authenticated `PUT .../upload-sessions/{session_id}/content` endpoint, which
  accepts a raw `application/octet-stream` body. The request-body middleware
  allows this endpoint to receive up to `MAX_UPLOAD_BYTES` while all other routes
  keep the smaller JSON limit. A `signed_url` method remains a future option for
  direct-to-object-storage deployments.
- **Metadata travels in an explicit manifest.** Because `.npy` files carry no
  channel metadata, the create-session request requires a `manifest`
  (`sampling_frequency_hz`, `channel_names`, optional `channel_units`). The
  manifest is stored on the session and copied to the dataset only after
  successful validation.
- **Object keys are server-generated** as
  `organisations/{organisation_id}/datasets/{dataset_id}/{uuid4}.npy` and are
  never returned to clients.
- **Untrusted NumPy data is never pickle-loaded.** Validation opens the array
  with `np.load(..., mmap_mode="r", allow_pickle=False)` and reads only the
  header, so large but malformed recordings are rejected without materialising
  data in API memory.

## Consequences

- Phase 5 is demonstrable end to end without the Phase 6 worker; the finalise
  endpoint is blocking and clients must treat it as such until the contract
  changes.
- The api-spec `202 Accepted` / `validating` example is deferred; changing it
  back later is a deliberate, test-backed contract change.
- Finalisation remains idempotent and the storage key stays server-side, keeping
  the existing security rules intact.
- A future direct-to-storage strategy can be added by returning a `signed_url`
  method without changing the session or dataset lifecycle.
