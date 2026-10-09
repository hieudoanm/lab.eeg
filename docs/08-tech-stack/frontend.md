## 4. Frontend

### 4.1 React

Use React for reusable, interactive interface components.

Responsibilities include:

- Study and dataset workflows.
- Form state.
- Signal channel selection.
- Time-range controls.
- Analysis job status.
- Result tables.
- Accessible loading, empty, and error states.

Keep domain logic separate from visual components wherever practical.

### 4.2 TypeScript

Use strict TypeScript settings.

Requirements:

- Enable `strict`.
- Avoid `any`.
- Represent API resources with explicit types.
- Model job states as a finite union of supported values.
- Narrow unknown error values before accessing properties.
- Validate untrusted server responses at appropriate boundaries.
- Keep API request and response definitions synchronised with the backend contract.

Avoid duplicating complex domain logic between the frontend and backend.

### 4.3 Next.js

Use Next.js as the frontend application framework.

Choose one routing approach and follow it consistently. The App Router is the recommended starting point.

Use client-side components only where interactivity requires them, including the signal explorer and interactive forms.

Keep scientific visualisation code out of server-rendered execution paths when the library depends on browser APIs.

Do not introduce a separate Node.js backend merely to proxy ordinary API calls. The tRPC Backend-for-Frontend described in 4.5 runs inside the Next.js process and is the only sanctioned exception, because it centralises session handling, organisation scoping, and typed contracts without adding a deployable service.

### 4.4 Styling

Use Tailwind CSS with a centralised design-token system.

Use daisyUI as the component layer on top of Tailwind CSS. daisyUI supplies accessible, themeable primitives (buttons, inputs, badges, cards, tables, steps, alerts, and so on) that the reusable components wrap, so styling stays consistent without hand-rolling every control.

Requirements:

- Reuse tokens from `09-design.md`.
- Map the EEG Lab palette to two daisyUI themes (`lab-light` and `lab-dark`, selectable at runtime with a persisted toggle); do not redefine colours in component code.
- Prefer reusable components for repeated interface patterns.
- Prefer daisyUI component classes over bespoke utility stacks for standard controls.
- Avoid scattered arbitrary values.
- Keep responsive behaviour explicit.
- Ensure focus states and accessible contrast.
- Avoid unnecessary inline styles.

A small accessible component layer is preferable to building an entire custom design system before the main workflows work.

### 4.5 Data fetching

Use a tRPC Backend-for-Frontend (BFF) that runs inside the Next.js server. The browser
never calls the Python API directly; it calls tRPC procedures, which run server-side and
forward authenticated requests upstream.

This is the documented architectural reason referenced by 4.3:

- The BFF is a thin, type-safe adapter in the same Next.js process, not a separate
  deployable Node.js service, so it adds no operational surface.
- React Server Components read data through a server-side tRPC caller, letting a page
  compose several upstream calls without exposing the REST API to the browser.
- Client components call the same procedures through `@trpc/react-query`, so server and
  client share one contract, one error model, and one organisation-scoping rule.
- The HttpOnly session cookie is read only on the server and forwarded as a bearer
  token, so credentials never reach client JavaScript.

Requirements:

- Define one procedure per user-visible operation; do not expose raw REST endpoints.
- Resolve the active organisation on the server from the verified session and the
  preference cookie; never trust an organisation identifier supplied by the client.
- Guard every upstream call so REST failures become tRPC errors with safe messages and
  the upstream request identifier available to server components.
- Use `@tanstack/react-query` for client-side caching, through tRPC and no other
  overlapping query/cache library.
- Revalidate the affected server-rendered paths after mutations.

The underlying typed REST client still owns request typing, error normalisation, request
cancellation, and authentication integration; tRPC procedures delegate to it.

### 4.6 Tauri

Use Tauri v2 to package the Next.js frontend as a native desktop application.

Responsibilities:

- Provide a native macOS, Windows, and Linux desktop experience.
- Use the system WebView (WebView2 on Windows, WebKit on macOS/Linux) for rendering.
- Expose minimal native APIs through Tauri plugins (shell, fs, dialog, notification, opener).
- Package the Next.js standalone output with a lightweight Rust HTTP server (axum) for production builds.
- Use the Next.js dev server directly during development.

Requirements:

- Keep the frontend codebase identical between web and desktop.
- Use `@tauri-apps/api` for any native integration.
- Configure Tauri capabilities to grant only the necessary permissions.
- Use the existing PWA manifest and icons for desktop app icons.
- Development: `pnpm tauri:dev` runs Next.js dev server + Tauri.
- Production: `pnpm tauri:build` builds Next.js standalone + Tauri bundle.

The Tauri configuration lives in `packages/frontend/src-tauri/`.

### 4.7 Plotly.js

Use Plotly.js for interactive time-series visualisation.

Requirements:

- Keep chart input bounded.
- Separate chart presentation data from raw source data.
- Support channel selection and time navigation.
- Avoid unnecessary chart reinitialisation.
- Handle empty data and invalid values explicitly.
- Provide accessible summaries or tabular alternatives.

The chart must not fetch unlimited data merely because a user zooms out or changes the display.

### 4.8 SEO, sharing, crawlers, and PWA

Treat public metadata and installability as part of the frontend platform rather
than as an afterthought. The public landing page and login page may be indexed
and shared; authenticated application routes and API routes must not be exposed
to search engines or crawlers.

#### Open Graph and social metadata

Define site-wide metadata in `packages/frontend/src/app/layout.tsx` using the
Next.js Metadata API:

- Set the default title, title template, description, canonical URL, locale,
  and site name.
- Configure Open Graph metadata for `website` pages.
- Use a committed 1200×630 PNG at
  `packages/frontend/public/og/og.png` for link previews, with its editable SVG
  source at `packages/frontend/public/og/og.svg`.
- Include equivalent Twitter `summary_large_image` metadata.
- Keep preview text factual and consistent with the product scope; do not imply
  clinical diagnosis or validated medical use.

#### Sitemap and robots policy

Implement `packages/frontend/src/app/sitemap.ts` and
`packages/frontend/src/app/robots.ts` so Next.js generates `/sitemap.xml` and
`/robots.txt`.

The sitemap should contain only public routes such as `/` and `/login`.
Authenticated routes may be listed only when they are intentionally public and
contain no private organisation, study, dataset, or result data. Never include
API endpoints or private resource URLs.

The robots policy must:

- Allow crawling of the public landing and login pages.
- Disallow `/app/` and `/api/`.
- Point to the canonical sitemap URL.
- Block known AI and large-scale training crawlers, including `GPTBot`,
  `ChatGPT-User`, `CCBot`, `anthropic-ai`, `Claude-Web`, and `PerplexityBot`,
  where the deployment's legal and product policy requires that restriction.

Robots rules are not an access-control mechanism. Authentication, authorisation,
and route protection must remain authoritative.

#### Human- and security-readable metadata

Serve the following static files from `packages/frontend/public/`:

- `humans.txt` with project maintainers, repository, technology stack, and
  security contact information.
- `.well-known/security.txt` following RFC 9116, with a valid security contact,
  policy URL, canonical URL, preferred language, and explicit expiry date.

Keep contact addresses and URLs aligned with the repository's `SECURITY.md` and
do not publish placeholder contacts in a production deployment.

#### Progressive Web App

Provide `packages/frontend/public/manifest.json` with:

- `name`, `short_name`, `description`, `start_url`, `scope`, and
  `display: "standalone"`.
- A theme/background colour pair matching `09-design.md`.
- Maskable app icons at the required common sizes, generated from the editable
  source in `packages/frontend/public/icons/icon.svg`.
- Shortcuts only for public or authenticated destinations whose route guards
  handle unauthorised users safely.

If offline caching is enabled, use a reviewed service-worker strategy and cache
only immutable static assets by default. Do not cache authenticated API
responses, organisation data, dataset contents, analysis results, signed URLs,
or session material without an explicit threat model and invalidation strategy.
The PWA must degrade safely to the ordinary browser experience when service
workers are unavailable.

Validate metadata, generated asset dimensions, manifest routes, robots rules,
and security.txt in CI or focused frontend tests. Regenerate derived PNG assets
from their SVG sources rather than editing generated files manually.

