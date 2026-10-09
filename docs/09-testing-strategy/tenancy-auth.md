## 9. Multi-Tenancy and Authorisation Tests

Tenant isolation is a critical security requirement.

Create at least two organisations with separate users, memberships, studies, datasets, and analysis jobs.

Test that a user from Organisation A cannot read or modify Organisation B's resources by:

- Changing the organisation ID in the URL.
- Substituting another organisation's study ID.
- Substituting another organisation's dataset ID.
- Substituting another organisation's analysis ID.
- Reusing an identifier obtained from a different context.
- Submitting a dataset from another organisation in an analysis request.
- Attempting to use another organisation's upload session.

Test all role boundaries:

- Owners can perform permitted administrative operations.
- Researchers can perform permitted scientific workflows.
- Viewers cannot modify data or submit analyses.
- Unauthenticated users cannot access protected resources.
- Disabled users or removed memberships cannot continue using previously granted access.

### Database-level integrity

Where composite tenant foreign keys are used, test that the database rejects cross-organisation relationships even when application-level validation is bypassed.

### Query scoping

Verify that organisation filtering is applied before pagination and aggregation. A correct permission check on a parent resource does not automatically make every child query safe.

