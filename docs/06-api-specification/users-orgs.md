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

