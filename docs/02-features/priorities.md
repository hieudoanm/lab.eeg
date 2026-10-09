## 3. Authentication and Organisation Context

**Priority:** P0

### Description

Users authenticate and work within an organisation. Organisation context determines which studies, datasets, jobs, and results they may access.

### User experience

- The application provides a clear sign-in experience.
- After authentication, the user sees their available organisations.
- The user selects an organisation or returns to the last valid selection.
- The application displays the active organisation consistently.
- If a session expires, the user receives a clear recovery path.
- If the user has no organisation membership, the application explains what they can do next.

### Behaviour

1. Authenticate the user.
2. Retrieve the user's identity and memberships.
3. Select an organisation for which membership has been verified.
4. Load organisation-scoped resources.
5. Include the organisation context in relevant API requests.
6. Independently verify membership and permissions on the server.

### Edge cases

- Invalid or expired session.
- Removed organisation membership.
- Organisation deleted or deactivated.
- User belongs to several organisations.
- User attempts to alter an organisation identifier in an API request.
- Authentication provider unavailable.

### Completion criteria

- Users cannot access private application resources without authentication.
- Changing the active organisation changes the visible resource scope.
- Direct API requests cannot bypass membership checks.
- Authentication errors are handled without exposing sensitive details.

