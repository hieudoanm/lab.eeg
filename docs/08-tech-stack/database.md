## 6. Database

### 6.1 PostgreSQL

Use PostgreSQL for durable application state.

Store:

- Organisations.
- Users and memberships.
- Studies.
- Dataset metadata.
- Dataset validation state.
- Analysis jobs.
- Analysis results.
- Audit events.
- Idempotency records where required.

Large raw EEG arrays should not normally be stored directly in PostgreSQL.

### 6.2 Explicit SQL

Use Psycopg 3 with its asynchronous interface for database access.

Prefer explicit SQL over a heavyweight ORM to demonstrate clear understanding of:

- Joins.
- Transactions.
- Constraints.
- Indexes.
- Query performance.
- Locking.
- Isolation.
- Data integrity.

Use repository modules to centralise query logic without hiding SQL behind excessive abstractions.

Requirements:

- Parameterise all values.
- Never interpolate untrusted values into SQL.
- Use transactions for multi-step state changes.
- Add constraints for important invariants.
- Review queries for organisation isolation.
- Add indexes based on expected access patterns.
- Use connection pooling with bounded connections.
- Test migrations against a real PostgreSQL instance in CI where practical.

### 6.3 Database migrations

Store migrations as paired, ordered `.up.sql` and `.down.sql` files under `packages/backend/migrations/versions/`. Apply them with the Psycopg-based runner using `make db-migrate` or `uv run python -m app.db.migrations upgrade`.

Every schema change must include a migration.

Requirements:

- Keep migration contents in SQL files rather than Python revision scripts.
- Give every upgrade migration a matching downgrade file.
- The runner tracks applied versions and checksums in `schema_migrations`, applies each file transactionally, and rejects modified applied migrations.
- Migrations must be repeatable in a clean environment.
- Migrations must be reviewed.
- Destructive changes must be treated carefully.
- Production deployment must apply migrations in a controlled step.
- Application rollback and schema rollback assumptions must be documented.

### 6.4 Database access rules

Every tenant-owned resource query must enforce the appropriate organisation scope.

Do not depend on the frontend to provide a trustworthy organisation identifier.

Consider PostgreSQL row-level security as an additional defence only if its session and transaction semantics are implemented and tested correctly. It must not substitute for application-level authorisation.

