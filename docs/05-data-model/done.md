## 22. Definition of Done

The data model is implemented when:

1. All core entities have migrations.
2. Primary and foreign keys are defined.
3. Important invariants have database constraints where possible.
4. Tenant-owned relationships enforce organisation consistency.
5. Query indexes support the primary workflows.
6. Job states and retry behaviour are documented.
7. Results retain dataset and algorithm provenance.
8. Upload and object-storage lifecycles are recoverable.
9. Idempotency and outbox mechanisms are tested where implemented.
10. Audit data avoids unnecessary sensitive information.
11. Migrations pass in CI.
12. Database integration tests cover critical integrity and isolation rules.
