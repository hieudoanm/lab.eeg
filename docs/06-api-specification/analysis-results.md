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

