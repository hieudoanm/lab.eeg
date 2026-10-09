## 20. Migration Strategy

Store versioned PostgreSQL migrations as paired `.up.sql` and `.down.sql` files under `packages/backend/migrations/versions/`. Run them with `make db-migrate` or `uv run python -m app.db.migrations upgrade`.

For each schema change:

1. Add the ordered SQL upgrade and corresponding downgrade files.
2. Review the SQL.
3. Check constraints and indexes.
4. Test upgrading an empty database.
5. Test upgrading a representative existing schema.
6. Run relevant integration tests.
7. Document any compatibility requirements.
8. Apply migrations through the deployment pipeline.

Avoid destructive schema changes in the same deployment that introduces code still depending on the old schema. Use expand-and-contract migrations for changes that require staged deployment.

Husky's pre-commit hook runs Ruff checks and formatting on staged Python source. Migration files are SQL and are validated by migration integration tests in CI.

