## 7. Organisation Isolation

Organisation isolation is a fundamental security boundary.

### 7.1 Trusted identity

The server obtains the user identity from the verified authentication mechanism.

It must not trust a client-supplied user ID as proof of identity.

### 7.2 Membership verification

Before accessing an organisation-scoped resource, the server verifies that the authenticated user has an appropriate membership.

### 7.3 Resource scoping

For a dataset belonging to a study, the application must verify the relationship between:

- Authenticated user.
- Organisation membership.
- Study.
- Dataset.
- Requested operation.

Checking only that the dataset ID exists is insufficient.

### 7.4 Defence in depth

Use:

- Server-side authorisation.
- Explicit organisation-scoped queries.
- Database constraints.
- Tests using multiple organisations.
- Carefully controlled service credentials.
- Optional database row-level security if implemented and tested correctly.

### 7.5 Cache isolation

Organisation-specific caches must be keyed by the appropriate organisation and user context.

When the active organisation changes, the frontend must invalidate relevant cached data.

A cache hit must never bypass a permission check that is required for the operation.

