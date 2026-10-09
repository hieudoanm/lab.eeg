## 11. Audit Trail

**Priority:** P1

### Description

The application records security-relevant actions to support troubleshooting and accountability.

### Initial events

- Study created, updated, or archived.
- Membership created, changed, or removed.
- Dataset uploaded.
- Dataset validation completed or failed.
- Analysis submitted, retried, completed, or failed.
- Access-control changes.

### Behaviour

Audit events must include the actor, organisation, action, resource, and timestamp when available.

Audit data must not contain access tokens, passwords, signed URLs, or unnecessary raw scientific data.

