# EEG Lab — API Specification

## 1. Purpose

This document defines the initial REST API for EEG Lab, an independent research-oriented neuroscience application for managing EEG datasets and running reproducible descriptive analyses.

The API is implemented with Python, FastAPI, and Pydantic. PostgreSQL is the authoritative source for application metadata, job state, and analysis results. Scientific recordings are stored in private object storage.

The API must support:

- Organisation-scoped access control.
- Study and dataset management.
- Secure recording uploads.
- Dataset validation and metadata retrieval.
- Asynchronous analysis submission.
- Analysis status polling and result retrieval.
- Auditability and reproducibility.
- Consistent errors, pagination, and idempotency.

This is an initial implementation contract. Endpoints may be added as the product evolves, but existing contracts should not change without a deliberate versioning decision.

## 2. API Conventions

### Base URL

Local development:

`http://localhost:8000/api/v1`

Production:

`https://<configured-api-host>/api/v1`

The production hostname must come from deployment configuration. Do not hard-code it into application source.

### HTTP methods

| Method   | Purpose                                     |
| -------- | ------------------------------------------- |
| `GET`    | Retrieve a resource or collection           |
| `POST`   | Create a resource or initiate an operation  |
| `PATCH`  | Partially update a resource                 |
| `DELETE` | Delete a resource when explicitly supported |

Use `202 Accepted` for operations accepted for asynchronous execution.

### Content types

- Requests containing JSON use `application/json`.
- JSON responses use `application/json`.
- File uploads use the upload protocol selected for the deployment. The local/development implementation uses an authenticated backend-mediated endpoint; production may use a short-lived signed object-storage URL.
- Large scientific recordings must not be embedded in JSON requests or responses.

### Identifiers

Use UUIDs for resource identifiers.

Example:

`/organisations/8f5d9c11-9b8d-4b65-9e10-4a83f7c0a112/studies`

### Timestamps

Return timestamps in ISO 8601 format with an explicit UTC offset, normally `Z`.

Example:

`2026-10-09T16:30:00Z`

### Naming

- Use `snake_case` for JSON field names.
- Use plural nouns for collection endpoints.
- Use stable machine-readable error codes.
- Do not expose database table names or internal implementation details unnecessarily.

## 3. Authentication and Authorisation

### Authentication

Protected endpoints require an authenticated user.

The backend must validate the identity token or session using the configured authentication mechanism. It must not trust user IDs supplied by the frontend.

Clients present a signed access token in one of two ways:

- `Authorization: Bearer <token>` for API and machine clients.
- An HttpOnly session cookie (default name `eeg-lab_session`) set by the API for browser clients, so the token is never exposed to frontend JavaScript.

When no valid credential is present the API returns `401 Unauthorized` with a `WWW-Authenticate: Bearer` header and an `authentication_required`, `invalid_token`, or `token_expired` error code.

#### `POST /auth/token`

Development-only. Mints a short-lived access token for the supplied development identity, provisions the matching user, and sets the session cookie.

This endpoint is disabled (returns `404 Not Found`) unless the API runs with `AUTH_MODE=development`, and is never available under `APP_ENV=production`.

Request:

```json
{
  "subject": "researcher@example.org",
  "identity_provider": "local",
  "display_name": "Ada Researcher",
  "email": "researcher@example.org"
}
```

All fields are optional and fall back to the configured development identity.

Response: `200 OK`

```json
{
  "access_token": "eyJ...",
  "token_type": "Bearer",
  "expires_at": "2026-10-09T17:30:00Z"
}
```

#### `POST /auth/logout`

Expires the session cookie and returns `204 No Content`. It always reports success, so callers need not hold a valid session.

### Organisation context

Organisation-scoped endpoints use the organisation identifier in the route.

For every request, the backend must:

1. Authenticate the caller.
2. Resolve the requested organisation.
3. Verify the caller's active membership.
4. Check the required role or permission.
5. Verify that all referenced resources belong to that organisation.
6. Execute the operation within the authorised scope.

The organisation ID in the URL is a selector, not proof of permission.

### Initial roles

| Capability                    | Owner |             Researcher | Viewer |
| ----------------------------- | ----: | ---------------------: | -----: |
| View studies and datasets     |   Yes |                    Yes |    Yes |
| Create and edit studies       |   Yes |                    Yes |     No |
| Initiate and finalise uploads |   Yes |                    Yes |     No |
| Submit analyses               |   Yes |                    Yes |     No |
| View analysis results         |   Yes |                    Yes |    Yes |
| Manage memberships and roles  |   Yes |                     No |     No |
| Archive studies               |   Yes | Yes, subject to policy |     No |

Operational permissions are separate from scientific data access. An operator role, if introduced, must not automatically grant access to research recordings or results.

### Resource isolation

Return `404 Not Found` for inaccessible organisation-scoped resources where appropriate, rather than disclosing that another organisation's resource exists.

Do not rely on frontend route protection alone.

## 4. Common Response Formats

### Successful response

A successful resource request returns the requested representation.

Example:

```json
{
  "id": "8f5d9c11-9b8d-4b65-9e10-4a83f7c0a112",
  "name": "Language EEG Pilot",
  "status": "active",
  "created_at": "2026-10-09T16:30:00Z",
  "updated_at": "2026-10-09T16:30:00Z"
}
```

### Paginated collection

Use a consistent pagination format.

```json
{
  "items": [],
  "next_cursor": null,
  "has_more": false
}
```

Requirements:

- Use cursor pagination for collections that may grow.
- Use a deterministic ordering, such as `created_at DESC, id DESC`.
- Bind cursors to the relevant ordering and query context.
- Never use a cursor as an authorisation mechanism.
- Apply organisation filtering before pagination.
- Set a maximum page size, initially 100 items.

The default page size should be configurable, with an initial default of 20.

### Error response

Use a consistent error envelope.

```json
{
  "error": {
    "code": "validation_error",
    "message": "The request contains invalid fields.",
    "details": [
      {
        "field": "sampling_frequency_hz",
        "code": "must_be_positive",
        "message": "Sampling frequency must be greater than zero."
      }
    ],
    "request_id": "req_example_123"
  }
}
```

Requirements:

- `code` is stable and machine-readable.
- `message` is safe for display.
- `details` may be empty.
- `request_id` helps correlate client reports with server logs.
- Never return stack traces, secrets, signed URLs, or raw internal exceptions.
- Do not expose another tenant's data in error details.

## 5. HTTP Status Codes

| Status                       | Meaning                                       |
| ---------------------------- | --------------------------------------------- |
| `200 OK`                     | Successful read or update                     |
| `201 Created`                | Resource created                              |
| `202 Accepted`               | Asynchronous operation accepted               |
| `204 No Content`             | Successful operation with no response body    |
| `400 Bad Request`            | Malformed or invalid request                  |
| `401 Unauthorized`           | Missing or invalid authentication             |
| `403 Forbidden`              | Authenticated caller lacks permission         |
| `404 Not Found`              | Resource missing or intentionally concealed   |
| `409 Conflict`               | State conflict or idempotency-key mismatch    |
| `413 Content Too Large`      | Upload exceeds the configured limit           |
| `415 Unsupported Media Type` | Unsupported upload format                     |
| `422 Unprocessable Entity`   | Request schema or semantic validation failure |
| `429 Too Many Requests`      | Rate limit exceeded                           |
| `500 Internal Server Error`  | Unexpected server failure                     |
| `503 Service Unavailable`    | Required service temporarily unavailable      |

Use `409 Conflict` for invalid state transitions and conflicting reuse of an idempotency key.

## 6. Endpoint Overview

All organisation-scoped routes begin with:

`/organisations/{organisation_id}`

| Method   | Endpoint                                                                                       | Purpose                                      |
| -------- | ---------------------------------------------------------------------------------------------- | -------------------------------------------- |
| `GET`    | `/me`                                                                                          | Return the current user and memberships      |
| `POST`   | `/organisations`                                                                               | Create an organisation and make caller owner |
| `GET`    | `/organisations`                                                                               | List organisations accessible to the caller  |
| `GET`    | `/organisations/{organisation_id}`                                                             | Get organisation details                     |
| `GET`    | `/organisations/{organisation_id}/summary`                                                     | Get organisation dashboard summary           |
| `GET`    | `/organisations/{organisation_id}/memberships`                                                 | List organisation members                    |
| `POST`   | `/organisations/{organisation_id}/memberships`                                                 | Add a member                                 |
| `PATCH`  | `/organisations/{organisation_id}/memberships/{membership_id}`                                 | Change a member's role                       |
| `DELETE` | `/organisations/{organisation_id}/memberships/{membership_id}`                                 | Remove a member                              |
| `GET`    | `/organisations/{organisation_id}/studies`                                                     | List studies                                 |
| `POST`   | `/organisations/{organisation_id}/studies`                                                     | Create a study                               |
| `GET`    | `/organisations/{organisation_id}/studies/{study_id}`                                          | Get a study                                  |
| `PATCH`  | `/organisations/{organisation_id}/studies/{study_id}`                                          | Update a study                               |
| `POST`   | `/organisations/{organisation_id}/studies/{study_id}/archive`                                  | Archive a study                              |
| `GET`    | `/organisations/{organisation_id}/studies/{study_id}/datasets`                                 | List study datasets                          |
| `POST`   | `/organisations/{organisation_id}/studies/{study_id}/datasets`                                 | Register dataset metadata                    |
| `GET`    | `/organisations/{organisation_id}/datasets/{dataset_id}`                                       | Get dataset metadata                         |
| `GET`    | `/organisations/{organisation_id}/datasets/{dataset_id}/signals`                               | Retrieve a bounded signal window             |
| `POST`   | `/organisations/{organisation_id}/datasets/{dataset_id}/upload-sessions`                       | Create an upload session                     |
| `PUT`    | `/organisations/{organisation_id}/datasets/{dataset_id}/upload-sessions/{session_id}/content`  | Upload recording content (backend-mediated)  |
| `POST`   | `/organisations/{organisation_id}/datasets/{dataset_id}/upload-sessions/{session_id}/finalise` | Finalise and validate an upload              |
| `GET`    | `/organisations/{organisation_id}/datasets/{dataset_id}/analyses`                              | List dataset analyses                        |
| `POST`   | `/organisations/{organisation_id}/datasets/{dataset_id}/analyses`                              | Submit an analysis                           |
| `GET`    | `/organisations/{organisation_id}/analyses`                                                    | List organisation analyses                   |
| `GET`    | `/organisations/{organisation_id}/analyses/{analysis_id}`                                      | Get analysis status                          |
| `GET`    | `/organisations/{organisation_id}/analyses/{analysis_id}/results`                              | Retrieve successful analysis results         |
| `POST`   | `/organisations/{organisation_id}/analyses/{analysis_id}/cancel`                               | Request cancellation                         |

Implement only endpoints required by the accepted MVP features. Do not create placeholder endpoints that return fake success responses.

The implementation also exposes unauthenticated liveness and readiness probes at `/health/live` and `/health/ready` (outside the `/api/v1` prefix). `/health/ready` returns `503` with its readiness representation when the database is unavailable.

## 7. Current User and Organisations

### `GET /me`

Returns the authenticated user and accessible organisation memberships.

Example response:

```json
{
  "id": "a1b2c3d4-1111-4222-8333-abcdefabcdef",
  "display_name": "Researcher",
  "email": "researcher@example.test",
  "memberships": [
    {
      "membership_id": "b7c1e2f3-2222-4333-8444-bcdefabcdefa",
      "organisation_id": "8f5d9c11-9b8d-4b65-9e10-4a83f7c0a112",
      "organisation_name": "EEG Lab Demo",
      "role": "researcher"
    }
  ]
}
```

Do not return authentication tokens or internal authentication-provider details.

### `GET /organisations`

Lists organisations the caller is authorised to access.

The backend derives this list from authenticated membership records, not from arbitrary client-supplied organisation IDs.

### `POST /organisations`

Creates an organisation and makes the authenticated caller its owner. The request contains a required `name` (1–200 characters) and an optional `slug` (up to 200 characters); success returns `201 Created` with the organisation details and the caller's `owner` role.

### Membership management

Membership management is restricted to organisation owners. Every mutation is recorded as an audit event in the same transaction as the change.

`POST /organisations/{organisation_id}/memberships` adds a member:

```json
{
  "identity_provider": "local",
  "subject": "researcher-42",
  "role": "researcher",
  "display_name": "Optional Display Name",
  "email": "optional@example.test"
}
```

The backend provisions a local user record for the `identity_provider` and `subject` pair when one does not exist. A user who is already a member returns `409 Conflict` with code `membership_exists`.

`PATCH /organisations/{organisation_id}/memberships/{membership_id}` changes a role using the same `role` field.

`DELETE /organisations/{organisation_id}/memberships/{membership_id}` removes a member and returns `204 No Content`.

An organisation must always retain at least one owner. Attempting to demote or remove the last owner returns `409 Conflict` with code `last_owner`. A membership identifier from another organisation is treated as `404 Not Found`.

### `GET /organisations/{organisation_id}/summary`

Returns organisation-scoped counts for dashboards. The response is computed from persisted records and never includes client-supplied values.

Response: `200 OK`

```json
{
  "study_count": 4,
  "active_study_count": 3,
  "dataset_count": 9,
  "ready_dataset_count": 7,
  "analysis_count": 21,
  "analyses_by_status": {
    "queued": 1,
    "running": 2,
    "succeeded": 15,
    "failed": 2,
    "cancelled": 1
  }
}
```

`active_study_count` counts studies that are not archived. `analyses_by_status` always contains all five lifecycle states, using `0` when a state has no jobs. The caller must be a member of the organisation; a foreign organisation is treated as `404 Not Found`.

## 8. Study Endpoints

### `POST /organisations/{organisation_id}/studies`

Creates a study.

Request:

```json
{
  "name": "Language EEG Pilot",
  "description": "Exploratory analysis of synthetic EEG recordings."
}
```

Validation:

- `name` is required and must not be blank.
- Trim surrounding whitespace.
- Apply a documented maximum length.
- Enforce a reasonable description length.
- Reject unsupported fields.

Response: `201 Created`

```json
{
  "id": "8f5d9c11-9b8d-4b65-9e10-4a83f7c0a112",
  "name": "Language EEG Pilot",
  "description": "Exploratory analysis of synthetic EEG recordings.",
  "status": "active",
  "created_at": "2026-10-09T16:30:00Z",
  "updated_at": "2026-10-09T16:30:00Z"
}
```

### `GET /organisations/{organisation_id}/studies`

Supported query parameters:

- `limit`: optional page size.
- `cursor`: optional pagination cursor.
- `status`: optional status filter.
- `search`: optional case-insensitive substring match on the study name.

The API must apply tenant and permission filters before returning results.

### `PATCH /organisations/{organisation_id}/studies/{study_id}`

Allows authorised users to update supported study fields.

Example:

```json
{
  "name": "Language EEG Pilot — Phase 2",
  "description": "Updated study description."
}
```

Use partial-update semantics. Distinguish omitted fields from explicitly provided `null` values.

### `POST /organisations/{organisation_id}/studies/{study_id}/archive`

Archives the study.

Response: `200 OK`

```json
{
  "id": "8f5d9c11-9b8d-4b65-9e10-4a83f7c0a112",
  "status": "archived",
  "archived_at": "2026-10-09T16:45:00Z"
}
```

Archiving must not silently delete source recordings or analysis results.

## 9. Dataset Endpoints

### `POST /organisations/{organisation_id}/studies/{study_id}/datasets`

Registers a dataset before upload.

Request:

```json
{
  "name": "Participant Demo 001",
  "description": "Synthetic EEG recording for application testing.",
  "format": "npy",
  "source_description": "Generated synthetic data",
  "metadata": { "signal_type": "eeg" }
}
```

Response: `201 Created`

```json
{
  "id": "2b4c6d8e-1111-4222-8333-abcdefabcdef",
  "study_id": "8f5d9c11-9b8d-4b65-9e10-4a83f7c0a112",
  "name": "Participant Demo 001",
  "format": "npy",
  "status": "pending_upload",
  "created_at": "2026-10-09T16:30:00Z"
}
```

The backend generates the dataset identifier and establishes organisation ownership. The client cannot choose an arbitrary storage key or mark a dataset as ready.

### `GET /organisations/{organisation_id}/datasets/{dataset_id}`

Returns dataset metadata and lifecycle state.

Example:

```json
{
  "id": "2b4c6d8e-1111-4222-8333-abcdefabcdef",
  "study_id": "8f5d9c11-9b8d-4b65-9e10-4a83f7c0a112",
  "name": "Participant Demo 001",
  "format": "npy",
  "status": "ready",
  "sampling_frequency_hz": 256.0,
  "channel_count": 8,
  "sample_count": 76800,
  "duration_seconds": 300.0,
  "channel_names": ["F3", "F4", "C3", "C4", "P3", "P4", "O1", "O2"],
  "checksum_sha256": "example_checksum_value",
  "validated_at": "2026-10-09T16:35:00Z"
}
```

The checksum above is illustrative, not a valid checksum to reuse.

Do not expose private object-storage keys or signed URLs through ordinary dataset responses.

### `GET /organisations/{organisation_id}/datasets/{dataset_id}/signals`

Returns a bounded, decimated window of a ready dataset's stored recording for charting. The read is display-only: it never feeds analysis and never mutates the stored recording.

Query parameters:

| Name            | Required | Bounds                          | Description                                              |
| --------------- | -------- | ------------------------------- | -------------------------------------------------------- |
| `start_seconds` | No       | `>= 0`, default `0`             | Start of the window, in seconds from the recording start |
| `end_seconds`   | Yes      | `> 0`                           | End of the window, in seconds; must be after `start`     |
| `channels`      | No       | comma-separated channel indices | Selected channels; defaults to the first channel (`0`)   |

Response: `200 OK`

```json
{
  "dataset_id": "2b4c6d8e-1111-4222-8333-abcdefabcdef",
  "sampling_frequency_hz": 256.0,
  "channel_count": 8,
  "sample_count": 76800,
  "duration_seconds": 300.0,
  "start_seconds": 1.0,
  "end_seconds": 2.0,
  "sample_start": 256,
  "sample_end": 512,
  "channel_indices": [0, 1],
  "points_per_channel": 256,
  "downsampled": false,
  "downsample_factor": 1,
  "time_seconds": [1.0, 1.00390625, 1.0078125],
  "channels": [
    {
      "channel_index": 0,
      "channel_name": "F3",
      "unit": "uV",
      "samples": [0.02, -0.11, 0.34]
    }
  ]
}
```

Validation and errors:

- The dataset must belong to the requested organisation; a foreign dataset is `404 Not Found`.
- The dataset must be `ready`; otherwise `409 Conflict` with code `dataset_not_ready`.
- A ready dataset whose recording object is missing returns `409 Conflict` with code `dataset_recording_missing`.
- A recording that cannot be parsed returns `409 Conflict` with code `dataset_recording_invalid`.
- An invalid window (for example `end_seconds <= start_seconds`, a window beyond the recording, or a window longer than the configured maximum) returns `422 Unprocessable Entity` with code `invalid_signal_window`.
- Non-integer channel values return `422 Unprocessable Entity` with code `invalid_signal_window`.
- A channel selection that resolves to no valid channels returns `422 Unprocessable Entity` with code `invalid_channel_selection`.

The response is bounded: the service decimates so that no more than the configured maximum number of points per channel is returned, setting `downsampled` and `downsample_factor` accordingly. Non-finite samples are reported as `null` so charts render a gap, and `samples` always align with the shared `time_seconds` axis. This endpoint feeds display only and must never be used as input to analysis.

## 10. Upload Endpoints


### `POST /organisations/{organisation_id}/datasets/{dataset_id}/upload-sessions`

Creates a new upload session for a dataset in `pending_upload` state.

Request:

```json
{
  "file_name": "synthetic_eeg.npy",
  "file_size_bytes": 1966080,
  "format": "npy",
  "expected_checksum_sha256": null,
  "manifest": {
    "sampling_frequency_hz": 256.0,
    "channel_names": ["F3", "F4", "C3", "C4", "P3", "P4", "O1", "O2"],
    "channel_units": ["uV", "uV", "uV", "uV", "uV", "uV", "uV", "uV"]
  }
}
```

The `manifest` supplies the metadata that is not embedded in the array. `sampling_frequency_hz` and `channel_names` are required; `channel_units` is optional but, when present, must have the same length as `channel_names`. `format` defaults to `npy`.

Response: `201 Created`

```json
{
  "upload_session_id": "3c5d7e9f-1111-4222-8333-abcdefabcdef",
  "status": "created",
  "upload_method": "backend",
  "upload_url": "/api/v1/organisations/.../datasets/.../upload-sessions/3c5d7e9f-1111-4222-8333-abcdefabcdef/content",
  "expires_at": "2026-10-09T17:00:00Z"
}
```

With `upload_method: "backend"` the client uploads bytes to the `upload_url` (an authenticated endpoint on the same origin). A future direct-to-object-storage deployment may instead return `upload_method: "signed_url"` with an absolute provider URL.

Security requirements:

- Verify the user can upload to the dataset's organisation and study.
- Apply upload size limits.
- Use a server-generated object key.
- Use short-lived, narrowly scoped upload authorisation.
- Never allow clients to select arbitrary bucket names or object keys.
- Do not log signed URLs or object keys.
- Restrict the accepted file types and upload methods.

### `PUT /organisations/{organisation_id}/datasets/{dataset_id}/upload-sessions/{session_id}/content`

Uploads the raw recording bytes for a backend-mediated session.

- The request body is the recording itself; `Content-Type` is `application/octet-stream`.
- Response: `204 No Content` on success.
- The body limit for this endpoint is `MAX_UPLOAD_BYTES`, not the smaller JSON limit applied to other routes.
- Returns `409 Conflict` (`upload_session_expired`, `upload_session_conflict`) when the session cannot accept content.
- Returns `413 Content Too Large` (`payload_too_large`) when the body exceeds `MAX_UPLOAD_BYTES`.

### `POST /organisations/{organisation_id}/datasets/{dataset_id}/upload-sessions/{session_id}/finalise`

Requests upload finalisation and validation.

Response: `200 OK`

```json
{
  "upload_session_id": "3c5d7e9f-1111-4222-8333-abcdefabcdef",
  "upload_method": "backend",
  "status": "completed",
  "dataset": {
    "id": "2b4c6d8e-1111-4222-8333-abcdefabcdef",
    "study_id": "8f5d9c11-9b8d-4b65-9e10-4a83f7c0a112",
    "name": "Participant Demo 001",
    "description": "Synthetic EEG recording for application testing.",
    "format": "npy",
    "status": "ready",
    "sampling_frequency_hz": 256.0,
    "channel_count": 8,
    "sample_count": 76800,
    "duration_seconds": 300.0,
    "channel_names": ["F3", "F4", "C3", "C4", "P3", "P4", "O1", "O2"],
    "channel_units": ["uV", "uV", "uV", "uV", "uV", "uV", "uV", "uV"],
    "checksum_sha256": "example_checksum_value",
    "file_size_bytes": 1966080,
    "source_description": "Generated synthetic data",
    "created_at": "2026-10-09T16:30:00Z",
    "updated_at": "2026-10-09T16:35:00Z",
    "validated_at": "2026-10-09T16:35:00Z"
  }
}
```

In the current implementation validation runs **synchronously** during finalisation, so the response reports the terminal state directly (`completed` + dataset `ready`). Phase 6 may move validation to a background worker and change this to `202 Accepted` with a `validating` dataset status; until then clients must treat this endpoint as blocking.

The endpoint verifies the uploaded object exists, matches the declared size, and (when supplied) matches the expected checksum. Finalisation is idempotent: re-finalising a completed session returns the existing ready dataset with `200 OK`.

Validation verifies:

- Actual file size.
- Checksum where available or required.
- File format.
- Array shape and orientation (`(channels, samples)`).
- Numeric data type; pickle-based object arrays are always rejected.
- Sampling frequency and metadata.
- Channel labels and count consistency.
- Absence of unsupported or unsafe file structures.

Only successful validation transitions a dataset to `ready`.

On a validation failure the endpoint returns `422 Unprocessable Entity` with code `invalid_recording`, the dataset transitions to `rejected`, the session to `failed`, and a `dataset.validation_failed` audit event is recorded. State conflicts return `409 Conflict` (`upload_session_expired`, `upload_content_missing`, `finalisation_in_progress`).

### Initial EEG format contract

For the initial `.npy` format:

- Use numeric arrays with documented supported dtypes.
- Define the expected orientation as `(channels, samples)`.
- Require channel metadata to match the first dimension.
- Require the sampling frequency to be finite and greater than zero.
- Reject arrays that exceed configured resource limits.
- Disable pickle-based object loading for untrusted uploads.
- Do not infer scientific meaning solely from the filename.

Use an accompanying validated JSON manifest if needed to supply metadata not embedded in the array.

## 11. Analysis Submission

### `POST /organisations/{organisation_id}/datasets/{dataset_id}/analyses`

Submits an analysis for asynchronous execution.

Headers:

- `Authorization`: required authentication.
- `Idempotency-Key`: required for the initial implementation of analysis submission.

Request:

```json
{
  "analysis_type": "descriptive_statistics",
  "algorithm_version": "1.0.0",
  "parameters": {
    "include_mean": true,
    "include_standard_deviation": true,
    "include_min_max": true,
    "include_peak_to_peak": true
  }
}
```

Validation:

- The dataset must belong to the requested organisation.
- The dataset must be `ready`.
- The caller must have permission to submit analyses.
- The analysis type must be supported.
- The requested algorithm version must be available.
- Parameters must conform to the Pydantic schema.
- File size, sample count, and other processing limits must be respected.

Response: `202 Accepted`

```json
{
  "id": "4d6e8f10-1111-4222-8333-abcdefabcdef",
  "dataset_id": "2b4c6d8e-1111-4222-8333-abcdefabcdef",
  "analysis_type": "descriptive_statistics",
  "algorithm_version": "1.0.0",
  "status": "queued",
  "requested_at": "2026-10-09T16:40:00Z"
}
```

### Idempotency behaviour

For the same authenticated user, organisation, operation, and idempotency key:

- An identical request must not create another logical analysis job.
- The API should return the original resource or a consistent representation of it.
- Reusing the key with a different canonical request hash must return `409 Conflict`.
- Concurrent duplicate requests must be safe.

The server must persist the job and its outbox event transactionally before returning acceptance.

## 12. Analysis Status and Results

### `GET /organisations/{organisation_id}/analyses`

Lists analysis jobs across every dataset in the organisation. The result is tenant-scoped and cursor-paginated, ordered by `requested_at` descending.

Query parameters:

- `cursor`: opaque pagination cursor from a previous response.
- `limit`: page size, `>= 1`; capped at the configured `MAX_PAGE_SIZE`.
- `status`: filter by lifecycle state (`queued`, `running`, `succeeded`, `failed`, `cancelled`).
- `dataset_id`: restrict to a single dataset.
- `study_id`: restrict to datasets belonging to a single study.

Response: `200 OK`, a paginated collection using the shared envelope:

```json
{
  "items": [
    {
      "id": "4d6e8f10-1111-4222-8333-abcdefabcdef",
      "dataset_id": "2b4c6d8e-1111-4222-8333-abcdefabcdef",
      "analysis_type": "descriptive_statistics",
      "algorithm_version": "1.0.0",
      "status": "succeeded",
      "attempt_count": 1,
      "max_attempts": 3,
      "requested_at": "2026-10-09T16:40:00Z",
      "started_at": "2026-10-09T16:40:02Z",
      "completed_at": "2026-10-09T16:40:05Z",
      "error": null
    }
  ],
  "next_cursor": null,
  "has_more": false
}
```

Ordering is deterministic (`requested_at` descending, then `id`) so pagination is stable. `dataset_id` and `study_id` from another organisation match nothing rather than disclosing foreign jobs. This list powers the organisation-wide analyses history view.

### `GET /organisations/{organisation_id}/analyses/{analysis_id}`


Returns the current job state.

Example:

```json
{
  "id": "4d6e8f10-1111-4222-8333-abcdefabcdef",
  "dataset_id": "2b4c6d8e-1111-4222-8333-abcdefabcdef",
  "analysis_type": "descriptive_statistics",
  "algorithm_version": "1.0.0",
  "status": "running",
  "attempt_count": 1,
  "requested_at": "2026-10-09T16:40:00Z",
  "started_at": "2026-10-09T16:40:02Z",
  "completed_at": null,
  "error": null
}
```

For a failed job, expose a safe error object.

```json
{
  "code": "analysis_execution_failed",
  "message": "The analysis could not be completed."
}
```

Do not expose raw stack traces or internal filesystem paths.

### `GET /organisations/{organisation_id}/analyses/{analysis_id}/results`

Returns results only when the analysis has succeeded.

If the analysis is still queued or running, return `409 Conflict` with a stable error code such as `results_not_ready`.

If it failed or was cancelled, return a safe error response explaining that no successful result is available.

Example successful response:

```json
{
  "analysis_id": "4d6e8f10-1111-4222-8333-abcdefabcdef",
  "dataset_id": "2b4c6d8e-1111-4222-8333-abcdefabcdef",
  "algorithm_version": "1.0.0",
  "result_schema_version": "1.0.0",
  "created_at": "2026-10-09T16:40:05Z",
  "channels": [
    {
      "channel_index": 0,
      "channel_name": "F3",
      "mean": 0.02,
      "standard_deviation": 0.98,
      "minimum": -3.1,
      "maximum": 3.4,
      "peak_to_peak": 6.5,
      "valid_sample_count": 76800
    }
  ],
  "provenance": {
    "input_checksum_sha256": "example_checksum_value",
    "sampling_frequency_hz": 256.0
  }
}
```

The values above are illustrative. The implementation must return calculated values, not hard-coded example output.

Scientific outputs must include appropriate units where known. If units are unknown, state that explicitly rather than inventing them.

## 13. Cancellation

### `POST /organisations/{organisation_id}/analyses/{analysis_id}/cancel`

Requests cancellation of an analysis.

Cancellation is cooperative: a worker may already be executing when the request arrives.

Rules:

- A queued job may be cancelled before execution.
- A running job may receive a cancellation request if supported.
- A succeeded job cannot be cancelled retroactively.
- Repeated cancellation requests must be safe.
- State transitions must be persisted.
- Workers must check the current state before committing results.

Return a response that reflects the actual persisted state. Do not report cancellation as complete while the job is still running.

If the initial implementation cannot reliably interrupt running computations, document that limitation and support cancellation only before execution begins.

## 14. Validation and Pydantic Schemas

Define request and response schemas explicitly.

Suggested schema modules:

```text
app/
  schemas/
    common.py
    auth.py
    organisations.py
    memberships.py
    studies.py
    datasets.py
    uploads.py
    analyses.py
    signals.py
    summary.py
    errors.py
```

Use Pydantic models for:

- Required and optional fields.
- String length limits.
- Numeric bounds.
- Supported status values.
- Nested metadata validation.
- Analysis parameter validation.
- Response serialisation.

Reject unknown request fields for write operations where practical. This reduces accidental acceptance of unsupported client input.

Do not use database models directly as public API response models.

## 15. Rate Limits and Resource Limits

Apply limits to resource-intensive operations.

Initial policies should cover:

- Upload size.
- Upload-session creation frequency.
- Analysis submissions per user or organisation.
- Maximum concurrent jobs.
- Maximum array dimensions and sample counts.
- Request body size.
- Pagination limits.
- Authentication failures.

Return `429 Too Many Requests` when a rate limit is exceeded, including `Retry-After` where appropriate.

Specific limits must be configurable and documented. Do not rely on application rate limiting as the only protection against excessive resource use. Rate limiting is not implemented in the current API; the checked-in OpenAPI artifact therefore does not advertise `429` responses until a limiter is wired into the request lifecycle.

## 16. Observability

Every request should have a correlation or request ID.

Log:

- Request ID.
- Route template and HTTP method.
- Response status.
- Duration.
- Relevant resource IDs.
- Safe error codes.

Avoid logging:

- Authentication tokens.
- Signed upload URLs.
- Raw EEG data.
- Unnecessary participant information.
- Full request bodies containing sensitive data.

Background workers should propagate correlation identifiers when available so API requests, outbox events, and analysis executions can be traced together.

The implementation exposes Prometheus metrics at `GET /metrics` (also accepts `HEAD`). This endpoint is unauthenticated and must be restricted to trusted monitoring networks by deployment controls.

## 17. API Documentation

FastAPI should generate an OpenAPI specification from the implemented routes and Pydantic schemas.
The checked-in artifact is based on that generated output and also documents authentication, the common error handlers, and mounted endpoints such as Prometheus metrics that are not fully represented by the default generator.

Requirements:

- Provide summaries and descriptions for endpoints.
- Document authentication.
- Document request and response schemas.
- Include error responses.
- Document supported enum values and lifecycle states.
- Keep examples consistent with the actual implementation.
- Make interactive API documentation available in development.
- Restrict or disable public interactive documentation in production according to deployment policy.
- Keep the checked-in implementation-derived specification at `docs/api/api-spec.json` in sync with the running FastAPI application.

OpenAPI output should be checked in CI for unintended contract changes where practical.

## 18. API Testing Requirements

Test at least:

### Authentication and permissions

- Unauthenticated requests are rejected.
- Invalid credentials are rejected.
- Viewers cannot perform write operations.
- Members cannot access another organisation's resources.
- Cross-tenant resource IDs do not bypass permission checks.
- Membership changes require the owner role.

### Resource management

- Study creation and updates.
- Dataset registration.
- Invalid state transitions.
- Dataset metadata validation.
- Pagination and filtering.
- Archive behaviour.

### Uploads

- Invalid formats are rejected.
- Oversized uploads are rejected.
- Expired upload sessions cannot be finalised.
- Invalid arrays are rejected.
- Duplicate finalisation requests are safe.
- Untrusted NumPy files cannot trigger pickle loading.

### Analyses

- Invalid parameters are rejected.
- Non-ready datasets cannot be analysed.
- Duplicate idempotency keys do not create duplicate jobs.
- Conflicting idempotency-key reuse returns `409`.
- Accepted jobs have durable outbox events.
- Failed jobs do not expose internal exceptions.
- Results are inaccessible before successful completion.
- Result provenance is retained.

Use pytest and HTTPX for backend API tests, with PostgreSQL integration tests for transaction and database behaviour.

Husky runs Ruff linting and formatting on staged Python API code. OpenAPI validation and API behaviour tests remain part of the broader local/CI quality checks rather than the pre-commit hook. Database migrations are stored separately as SQL files.

## 19. Definition of Done

The API is ready for the MVP when:

1. Implemented endpoints match this contract.
2. Authentication and organisation membership checks are enforced server-side.
3. Request and response schemas are explicit.
4. Errors follow the common format.
5. Pagination is deterministic and tenant-scoped.
6. Uploads are validated before datasets become ready.
7. Analysis jobs are asynchronous and idempotent.
8. Job creation and dispatch are reliable.
9. Results include versioned provenance.
10. Sensitive storage details are not exposed.
11. OpenAPI documentation matches the implementation.
12. Critical security, validation, and lifecycle cases are covered by tests.
