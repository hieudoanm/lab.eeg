## 3. Authentication and Authorisation

### Authentication

Protected endpoints require an authenticated user.

The backend must validate the identity token or session using the configured authentication mechanism. It must not trust user IDs supplied by the frontend.

Clients present a signed access token in one of two ways:

- `Authorization: Bearer <token>` for API and machine clients.
- An HttpOnly session cookie (default name `eeg-lab_session`) set by the API for browser clients, so the token is never exposed to frontend JavaScript.

When no valid credential is present the API returns `401 Unauthorized` with a `WWW-Authenticate: Bearer` header and an `authentication_required`, `invalid_token`, or `token_expired` error code.

#### `POST /auth/token`

Development-only. Mints a short-lived access token for the supplied development identity, provisions the matching user, and sets the session cookie.

This endpoint is disabled (returns `404 Not Found`) unless the API runs with `AUTH_MODE=development`, and is never available under `APP_ENV=production`.

Request:

```json
{
  "subject": "researcher@example.org",
  "identity_provider": "local",
  "display_name": "Ada Researcher",
  "email": "researcher@example.org"
}
```

All fields are optional and fall back to the configured development identity.

Response: `200 OK`

```json
{
  "access_token": "eyJ...",
  "token_type": "Bearer",
  "expires_at": "2026-10-09T17:30:00Z"
}
```

#### `POST /auth/logout`

Expires the session cookie and returns `204 No Content`. It always reports success, so callers need not hold a valid session.

### Organisation context

Organisation-scoped endpoints use the organisation identifier in the route.

For every request, the backend must:

1. Authenticate the caller.
2. Resolve the requested organisation.
3. Verify the caller's active membership.
4. Check the required role or permission.
5. Verify that all referenced resources belong to that organisation.
6. Execute the operation within the authorised scope.

The organisation ID in the URL is a selector, not proof of permission.

### Initial roles

| Capability                    | Owner |             Researcher | Viewer |
| ----------------------------- | ----: | ---------------------: | -----: |
| View studies and datasets     |   Yes |                    Yes |    Yes |
| Create and edit studies       |   Yes |                    Yes |     No |
| Initiate and finalise uploads |   Yes |                    Yes |     No |
| Submit analyses               |   Yes |                    Yes |     No |
| View analysis results         |   Yes |                    Yes |    Yes |
| Manage memberships and roles  |   Yes |                     No |     No |
| Archive studies               |   Yes | Yes, subject to policy |     No |

Operational permissions are separate from scientific data access. An operator role, if introduced, must not automatically grant access to research recordings or results.

### Resource isolation

Return `404 Not Found` for inaccessible organisation-scoped resources where appropriate, rather than disclosing that another organisation's resource exists.

Do not rely on frontend route protection alone.

