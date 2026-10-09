## 5. Phase 1 — Backend Foundation

### Objectives

Create the application API and persistent database layer.

### Tasks

- [x] Create the FastAPI application.
- [x] Add configuration management.
- [x] Define development, test, and production settings.
- [x] Add structured logging.
- [x] Add request IDs.
- [x] Configure PostgreSQL connectivity using Psycopg 3.
- [x] Implement explicit parameterised SQL access patterns.
- [x] Add connection pooling.
- [x] Configure versioned SQL migrations and runner.
- [x] Create the initial schema.
- [x] Add `/health/live`.
- [x] Add `/health/ready`.
- [x] Add error response models.
- [x] Add pytest and HTTPX.
- [x] Add PostgreSQL integration tests.

### Database access conventions

- Use parameterised SQL for all untrusted values.
- Never interpolate user-controlled strings into SQL statements.
- Keep transaction boundaries explicit.
- Avoid holding database transactions open while performing long computations or remote object-storage operations.
- Do not share one database connection concurrently across unrelated asynchronous tasks.
- Keep SQL and persistence logic separated from route handlers.

### Acceptance criteria

- The API starts locally.
- PostgreSQL connectivity is verified.
- Migrations can build a fresh database.
- Health endpoints behave correctly.
- A database integration test passes against real PostgreSQL.
- An intentionally failing dependency causes readiness to fail appropriately.

