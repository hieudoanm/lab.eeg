# 0005 — tRPC Backend-for-Frontend

- Status: Accepted
- Date: 2026-10-10
- Phase: Frontend data layer

## Context

`docs/08-tech-stack.md` §4.5 previously specified a single typed REST client used
directly by the frontend, and §4.3 discouraged introducing a Node.js backend
merely to proxy ordinary API calls. In practice the frontend had grown a set of
Server Actions and duplicated fetching concerns: session/tenancy resolution, error
normalisation, and organisation scoping were re-implemented per screen, and server
components used ad-hoc `fetch` calls that bypassed the shared types.

The Python API must remain the single source of truth for domain rules, tenancy,
and scientific validation (`docs/05-architecture.md` §7). Any frontend change must
preserve that boundary, keep the HttpOnly session cookie server-side, and avoid
adding another deployable service.

## Decision

- **Use an in-process tRPC Backend-for-Frontend.** The BFF lives inside the
  Next.js server (`src/server/api`) and exposes one procedure per user-visible
  operation. It is not a separate Node.js service, so it does not add a deployable
  surface or violate §4.3.
- **Server components read through a server caller.** A React Server Component
  resolves the session via `requireSession()` and calls procedures through
  `appRouter.createCaller(...)`, so a page can compose several upstream calls in
  one place without exposing the REST API to the browser.
- **Client components use the same procedures.** Interactive forms and controls
  call `@trpc/react-query` hooks against the shared `AppRouter` type, so server
  and client share one contract, one error model, and one scoping rule.
- **Organisation scope is resolved on the server.** Procedures derive the active
  organisation from the verified session and the preference cookie; no procedure
  accepts a client-supplied organisation identifier.
- **The session cookie never reaches client JavaScript.** The BFF reads the
  HttpOnly `eeg-lab_session` cookie, exchanges it for a bearer token, and calls
  the REST API through the existing typed `ApiClient`.
- **REST failures map to tRPC errors.** A guard converts `ApiError` into
  `TRPCError` with the original error attached as the cause, so server components
  can still surface the upstream request identifier for support.
- **React Query is the only client cache.** Client-side state uses
  `@tanstack/react-query` through tRPC; no overlapping query/cache library is
  introduced. Mutations revalidate the affected server-rendered paths.

## Consequences

- `docs/08-tech-stack.md` §4.3 and §4.5 are updated: the tRPC BFF is the single
  documented exception to the "do not proxy ordinary API calls" rule, and §4.5
  now records the requirements above.
- `docs/05-architecture.md` describes the BFF as the entry point for
  browser-originated requests ahead of the Python API request lifecycle.
- Server Actions (`src/app/actions.ts`) are removed; screens read and mutate
  through procedures, and existing component tests target tRPC hooks.
- The frontend gains tRPC, React Query, and Zod dependencies, and the vitest
  coverage thresholds continue to apply to the new server modules.
- The REST contract, tenancy enforcement, and scientific validation in the Python
  API are unchanged; the BFF only adapts transport and typing.
