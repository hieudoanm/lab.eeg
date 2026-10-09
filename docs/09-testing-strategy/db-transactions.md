## 10. Database and Transaction Tests

Use real PostgreSQL for tests that depend on PostgreSQL behaviour.

Test:

- Fresh schema creation.
- SQL migration upgrade execution.
- Unique membership constraints.
- Foreign key enforcement.
- Composite tenant constraints.
- Transaction rollback.
- Concurrent updates.
- Idempotency-key uniqueness.
- Outbox creation in the same transaction as job creation.
- Result persistence and job completion.
- Last-owner protection.
- Pagination ordering.
- Archival behaviour.

### Transactional job creation

A job submission must not produce a committed job without the corresponding durable dispatch event.

Test the transaction boundary:

1. Begin the transaction.
2. Create the analysis job.
3. Create its outbox event.
4. Commit both together.
5. Verify both records exist.

If either operation fails, verify that neither record is left in a partially committed state.

### Migration tests

Test migrations from:

- An empty database.
- The previous released schema, when applicable.
- A representative database containing existing rows.

For incompatible schema changes, verify the migration plan before deployment.

