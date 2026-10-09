## 13. Database Transactions and Integrity

Use PostgreSQL transactions for operations that must be atomic within the relational database.

Examples:

- Creating a study and related initial records.
- Creating a job and its outbox event.
- Persisting a result and marking the job successful.
- Updating membership and recording the corresponding audit event.

Use constraints for invariants that should remain true regardless of which application code path performs a write.

Do not assume that a successful object-storage operation can be rolled back by a PostgreSQL transaction.

Use compensating operations and reconciliation where multiple services participate in a workflow.

