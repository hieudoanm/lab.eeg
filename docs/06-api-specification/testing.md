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

