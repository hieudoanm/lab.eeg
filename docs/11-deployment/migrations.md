## 9. Database Migrations

Use versioned `.up.sql` and `.down.sql` files under `packages/backend/migrations/versions/` for schema changes. Apply pending SQL migrations with `uv run python -m app.db.migrations upgrade`.

### Deployment order

A typical release should:

1. Build and test the application images.
2. Review the migration plan.
3. Apply a backward-compatible migration.
4. Deploy compatible application code.
5. Run smoke tests.
6. Verify health and error metrics.
7. Retain a documented recovery option.

For changes requiring multiple releases, use expand-and-contract migrations:

1. Add the new schema while retaining compatibility.
2. Deploy code that supports the transition.
3. Migrate data if required.
4. Switch reads and writes.
5. Remove obsolete schema in a later release.

Do not assume rolling back an application image will automatically reverse a database migration safely.

Avoid destructive automatic schema changes during application startup.

