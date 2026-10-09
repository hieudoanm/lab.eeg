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

