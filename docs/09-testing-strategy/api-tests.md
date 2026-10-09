## 8. API Tests

Test endpoint behaviour using FastAPI's supported test integration with HTTPX.

### Study endpoints

- Valid study creation returns `201`.
- Missing or blank names are rejected.
- Unauthorised creation is rejected.
- Updating an archived study follows the documented policy.
- Pagination returns deterministic results.
- Invalid cursors are rejected safely.

### Dataset endpoints

- Dataset creation requires a valid parent study.
- A study from another organisation cannot be referenced.
- Unsupported file formats are rejected.
- Invalid metadata is rejected.
- Dataset state cannot be set arbitrarily by a client.
- A dataset cannot become ready before validation succeeds.

### Analysis endpoints

- Non-ready datasets cannot be analysed.
- Invalid analysis parameters are rejected.
- Duplicate submission with the same idempotency key does not create duplicate logical jobs.
- Reusing an idempotency key with different input returns `409`.
- A job's status reflects persisted state.
- Results are unavailable before successful completion.
- Failed jobs do not expose internal exceptions.
- Result provenance is returned correctly.

### Error responses

Verify that errors consistently include:

- A stable error code.
- A safe human-readable message.
- Field details where applicable.
- A request ID.

Do not assert on incidental internal exception wording.

