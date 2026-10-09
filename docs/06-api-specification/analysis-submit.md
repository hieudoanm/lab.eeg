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

