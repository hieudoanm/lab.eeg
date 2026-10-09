## 10. Organisation Membership Management

**Priority:** P1

### Description

Organisation owners manage access to their organisation.

### Initial capabilities

- List members.
- Inspect each member's role.
- Invite or add a member through the chosen identity mechanism.
- Change roles where permitted.
- Remove membership.
- Preserve the last-owner invariant.

If invitations are outside the initial MVP, the UI should not display non-functional invitation controls.

### Security

Every membership change must be authorised server-side and recorded as an audit event.

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
