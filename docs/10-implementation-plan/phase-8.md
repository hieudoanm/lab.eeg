## 12. Phase 8 — Reliability, Security, and Quality

### Objectives

Harden the complete vertical slice.

### Tasks

- [x] Review organisation isolation.
- [x] Review authentication and permissions.
- [x] Add request and resource limits.
- [x] Review upload security.
- [ ] Review signed URL handling.
- [x] Test transaction rollback behaviour.
- [x] Test duplicate event delivery.
- [x] Test stale-job recovery.
- [x] Add structured application and worker logs.
- [x] Add basic metrics.
- [x] Add correlation IDs.
- [x] Add database backup and recovery guidance.
- [x] Scan dependencies and container images.
- [x] Review secrets handling.
- [x] Add end-to-end tests.
- [x] Document known limitations.

### Acceptance criteria

- Critical security tests pass.
- Failures produce actionable logs without leaking secrets.
- Background jobs can recover from expected transient failures.
- Database and object-storage responsibilities are documented.
- The system has a tested recovery path for its initial deployment.
- The complete workflow passes automated end-to-end tests.

