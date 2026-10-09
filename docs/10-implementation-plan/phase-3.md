## 7. Phase 3 — Identity and Multi-Tenancy

### Objectives

Enforce secure organisation-scoped access before introducing real data workflows.

### Tasks

- [x] Choose the authentication approach.
- [x] Implement token or session validation.
- [x] Map authenticated identities to local user records.
- [x] Implement organisation membership lookup.
- [x] Implement role-based permission checks.
- [x] Add organisation-scoped query helpers.
- [x] Add tenant consistency constraints.
- [x] Add membership management for owners.
- [x] Add audit events for membership changes.
- [x] Test cross-organisation access attempts.

### Required roles

- `owner`
- `researcher`
- `viewer`

An operator role may be added later, but operational access must not automatically imply scientific data access.

### Security acceptance criteria

- Unauthenticated users cannot access protected endpoints.
- A viewer cannot submit analyses or modify datasets.
- A user cannot access another organisation's resources by changing a URL identifier.
- Referenced resources are checked for tenant consistency.
- Membership changes require appropriate permissions.
- Security-sensitive operations are auditable.

Do not move to real research data until these controls have been tested.

