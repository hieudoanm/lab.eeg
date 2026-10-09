## 21. Database Testing Requirements

Test at least:

- Unique membership per organisation and user.
- Valid membership roles.
- Study and dataset organisation consistency.
- Valid dataset states.
- Required metadata for ready datasets.
- Valid analysis job states.
- Job and result relationship integrity.
- Duplicate idempotency key behaviour.
- Transactional outbox creation.
- Last-owner protection.
- Audit event persistence.
- Organisation-scoped reads and writes.
- Concurrent job claiming.
- Migration correctness.

Use a real PostgreSQL instance for tests that depend on PostgreSQL-specific constraints, SQL semantics, transactions, or locking.

