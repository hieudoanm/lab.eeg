# 0001 — Authentication approach

- Status: Accepted
- Date: 2026-10-09
- Phase: 3 (Identity and Multi-Tenancy)

## Context

EEG Lab requires server-enforced authentication and organisation-scoped
authorisation (FR-001, FR-002, FR-005, NFR-001, NFR-002). The demonstration must
run locally without an external identity provider, while production must not be
able to enable development-only authentication accidentally (FR-006, NFR-008).

The schema already models users keyed by `(identity_provider,
external_subject)`, organisations, and memberships with the `owner`,
`researcher`, and `viewer` roles.

## Decision

Use signed JSON Web Tokens (JWT) as access tokens validated by the API, carried
either as a bearer token or in an HttpOnly session cookie:

- The API validates tokens presented in the `Authorization: Bearer` header or,
  for browser clients, in an HttpOnly session cookie (`AUTH_COOKIE_NAME`,
  default `eeg-lab_session`). The token's `sub`/`idp` claims are mapped to a
  local user record; client-supplied user or organisation identifiers are never
  trusted for identity.
- `AUTH_MODE=development` enables `POST /api/v1/auth/token`, which mints a
  short-lived HS256 token for a development subject, provisions the matching
  user record, and sets the HttpOnly session cookie on the response. This
  endpoint is only registered/usable when development auth is enabled and is
  never served under `APP_ENV=production`.
- A development user who belongs to no organisation is given a seeded owner
  organisation (`DEV_ORGANISATION_NAME`) on first sign-in so the workspace is
  usable locally (FR-006). This is idempotent, skips disabled accounts, and is
  only reachable through the development token endpoint.
- `POST /api/v1/auth/logout` clears the session cookie and returns `204`.
- `AUTH_MODE=oidc` is the production mode. The same validator verifies tokens
  issued by the configured provider using `AUTH_TOKEN_SECRET` (HMAC) or
  `AUTH_PUBLIC_KEY` (`RS*`/`ES*`/`PS*`), checked against `AUTH_TOKEN_ISSUER` and
  `AUTH_TOKEN_AUDIENCE`.
- The session cookie is `HttpOnly`, `SameSite=Lax` by default, and `Secure`
  automatically in production. `SameSite=None` is rejected unless the cookie is
  secure.
- Production settings fail closed at startup when `AUTH_MODE=development`, when
  the HMAC secret is the built-in default or shorter than 32 characters, or when
  an asymmetric algorithm is configured without a public key.

Authorisation is enforced in the API layer, not the frontend:

- `GET /api/v1/me` returns the caller and their memberships.
- Organisation-scoped routes resolve a verified `OrganisationContext` (caller,
  organisation, membership). A missing membership is reported as `404 Not
Found` so tenant existence is not disclosed (FR-012).
- Membership management requires the `owner` role (FR-008, FR-009, FR-010).
- The last-owner invariant is enforced transactionally: membership rows for an
  organisation are locked (`SELECT ... FOR UPDATE`) before counting active
  owners. Only owners whose user account is not disabled count (FR-011).

## Consequences

- Local development and tests can obtain real, verifiable tokens without an
  external provider, so the same authorisation code paths are exercised.
- Rotating to an external provider is a configuration change rather than a code
  change; a full JWKS/RS256 discovery flow remains a deployment-phase task and
  must be added before real production use.
- The development token endpoint must remain impossible to enable in
  production; `Settings.model_post_init` enforces this and tests assert it.
- Role changes and removals emit audit events (`membership.created`,
  `membership.role_changed`, `membership.removed`) in the same transaction as
  the mutation.
- Browser clients keep the token out of JavaScript: the Next.js server reads the
  HttpOnly cookie (`lib/session.ts`) and forwards it to the API, and the token
  value is never exposed to the browser bundle.

## Alternatives considered

- **Bearer tokens in browser JavaScript storage**: simpler to wire, but exposes
  the token to any script and requires manual expiry handling; rejected once the
  frontend needed cross-request persistence.
- **Opaque server-side sessions in our own store**: avoids JWTs but adds session
  storage and lookup scope that the JWT approach avoids while identity remains
  delegated to a provider. Revisit if server-side revocation becomes a
  requirement; the cookie transport is already in place.
- **Trusting a frontend-supplied user id**: rejected; violates FR-005 and
  NFR-001.
- **Password authentication**: rejected; out of scope for a demonstration where
  identity is delegated to a provider.
