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

