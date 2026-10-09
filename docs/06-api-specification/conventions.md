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

