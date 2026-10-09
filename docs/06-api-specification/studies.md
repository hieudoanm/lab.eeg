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

