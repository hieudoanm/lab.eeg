## 3. Authentication and Organisation Access

### US-001: Sign in

**As a** user,  
**I want to** sign in securely,  
**so that** I can access the organisations and studies available to me.

Acceptance criteria:

- Given valid credentials, when I sign in, then I receive an authenticated session.
- Given invalid credentials, when I sign in, then I receive a safe error message.
- Given an expired session, when I request a protected resource, then the application asks me to authenticate again or renews the session through the supported mechanism.
- Given an unauthenticated request to a protected API endpoint, then the API rejects the request.
- Authentication errors do not disclose whether another user's account exists unless the chosen identity provider explicitly permits that behaviour.

Priority: P0.

### US-002: Select an organisation

**As a** user who belongs to multiple organisations,  
**I want to** select my active organisation,  
**so that** I can work with the correct resources.

Acceptance criteria:

- Given several organisation memberships, when I select an organisation, then the application loads resources belonging to that organisation.
- Given no membership in the requested organisation, when I attempt to access it, then the server denies access.
- Changing the active organisation clears or invalidates cached organisation-specific data.
- Organisation switching does not expose data from the previously selected organisation.

Priority: P0.

### US-003: Enforce role permissions

**As an** organisation owner,  
**I want to** control what members can do,  
**so that** access matches their responsibilities.

Acceptance criteria:

- Given a viewer, when they attempt a write operation, then the API rejects it.
- Given a researcher, when they perform an allowed study or dataset operation, then the API permits it.
- Given an unauthorised user, when they call an API directly rather than using the UI, then the same permission rules apply.
- The frontend hides or disables unavailable actions where appropriate, but server-side authorisation remains authoritative.

Priority: P0.

