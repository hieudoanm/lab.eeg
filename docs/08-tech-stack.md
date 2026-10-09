# EEG Lab — Technology Stack

## 1. Purpose

This document defines the technologies, libraries, infrastructure, and development tools used to build EEG Lab.

The stack should demonstrate production-oriented full-stack engineering for a neuroscience research platform. It should favour maintainability, explicit contracts, testability, and reliable data processing over unnecessary complexity.

The implementation must remain consistent with:

- `01-overview.md`
- `02-requirements.md`
- `04-features.md`
- `03-user-stories.md`
- `09-design.md`

## 2. Technology Principles

1. Use mature, actively maintained technologies.
2. Prefer explicit types and validation at system boundaries.
3. Keep the frontend and backend independently testable.
4. Use asynchronous I/O where it improves throughput.
5. Use PostgreSQL for durable relational state.
6. Keep large scientific files outside the relational database.
7. Use background workers for computationally expensive operations.
8. Keep database access explicit and reviewable.
9. Make the application reproducible through automated setup and infrastructure configuration.
10. Avoid adding dependencies without a clear product or engineering benefit.

## 3. Stack Summary

| Layer                  | Technology                                        | Responsibility                                         |
| ---------------------- | ------------------------------------------------- | ------------------------------------------------------ |
| Frontend               | React, TypeScript                                 | Interactive research interface                         |
| Frontend framework     | Next.js                                           | Routing, application structure, production builds      |
| Frontend desktop       | Tauri v2                                          | Native desktop app wrapper for WebView                 |
| Frontend data layer    | tRPC, React Query                                 | Typed Backend-for-Frontend and client state            |
| Styling                | Tailwind CSS v4 + daisyUI v5                      | Responsive styling, design tokens, and UI primitives   |
| UI components          | Accessible reusable React components              | Forms, tables, navigation, dialogs                     |
| Signal visualisation   | Plotly.js                                         | EEG time-series exploration                            |
| Backend                | Python, FastAPI                                   | REST API and business logic                            |
| Validation             | Pydantic                                          | Request, response, and configuration validation        |
| Database               | PostgreSQL                                        | Users, organisations, studies, datasets, jobs, results |
| Database driver        | Psycopg 3 asynchronous API                        | Explicit asynchronous PostgreSQL access                |
| Database migrations    | PostgreSQL `.sql` files and a Psycopg runner      | Versioned, reviewable schema changes                   |
| Background jobs        | Celery                                            | Asynchronous analysis processing                       |
| Queue and broker       | Redis                                             | Job delivery and coordination                          |
| Scientific computing   | NumPy                                             | Numerical operations on signal arrays                  |
| EEG file support       | NumPy `.npy` plus JSON manifest initially         | Recording storage and metadata                         |
| Object storage         | Google Cloud Storage                              | Private scientific files and artefacts                 |
| Containerisation       | Docker                                            | Reproducible application and worker environments       |
| Cloud infrastructure   | Google Cloud Platform                             | Hosting and managed infrastructure                     |
| Infrastructure as code | Terraform                                         | Reproducible infrastructure provisioning               |
| CI/CD                  | GitHub Actions                                    | Testing, quality checks, and deployment                |
| Frontend tests         | Vitest, React Testing Library                     | Component and interaction testing                      |
| Browser tests          | Playwright                                        | End-to-end workflows                                   |
| Backend tests          | pytest, HTTPX                                     | Unit and API integration testing                       |
| Python quality         | Ruff, mypy                                        | Linting, formatting, and static type checking          |
| TypeScript quality     | ESLint, TypeScript compiler                       | Linting and type checking                              |
| Observability          | Structured logs and Google Cloud monitoring tools | Operational diagnostics                                |

Versions should be pinned through the project's dependency-management files and updated deliberately.

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

## 5. Backend

### 5.1 Python

Use a supported stable Python release compatible with the selected dependency set.

Python is suitable for the API, scientific data processing, and integration with established numerical libraries.

Pin the Python version in project tooling, Docker images, and CI configuration.

### 5.2 FastAPI

Use FastAPI for the REST API.

Responsibilities:

- Authentication integration.
- Request validation.
- Authorisation.
- Organisation-scoped resource access.
- Business rules.
- Dataset metadata management.
- Analysis job submission.
- Job and result retrieval.
- OpenAPI generation.
- Health and readiness endpoints.

Route handlers should remain thin. Put substantial business logic into service modules and persistence logic into repository modules.

### 5.3 Pydantic

Use Pydantic for:

- API request models.
- API response models.
- Environment and application configuration.
- Metadata validation.
- Analysis parameter validation.

Define separate request and response models where their security or lifecycle requirements differ.

Do not expose internal database records directly as public API responses.

### 5.4 Asynchronous execution

Use asynchronous FastAPI route handlers for I/O-bound operations.

Do not run expensive numerical computations directly on the event loop.

Delegate computational work to Celery workers. CPU-intensive work should not block the API process.

### 5.5 Error handling

Use consistent API error responses with:

- A stable error code.
- A human-readable message.
- Optional structured field errors.
- A request identifier where appropriate.

Do not return stack traces, SQL statements, credentials, or internal storage paths.

## 6. Database

### 6.1 PostgreSQL

Use PostgreSQL for durable application state.

Store:

- Organisations.
- Users and memberships.
- Studies.
- Dataset metadata.
- Dataset validation state.
- Analysis jobs.
- Analysis results.
- Audit events.
- Idempotency records where required.

Large raw EEG arrays should not normally be stored directly in PostgreSQL.

### 6.2 Explicit SQL

Use Psycopg 3 with its asynchronous interface for database access.

Prefer explicit SQL over a heavyweight ORM to demonstrate clear understanding of:

- Joins.
- Transactions.
- Constraints.
- Indexes.
- Query performance.
- Locking.
- Isolation.
- Data integrity.

Use repository modules to centralise query logic without hiding SQL behind excessive abstractions.

Requirements:

- Parameterise all values.
- Never interpolate untrusted values into SQL.
- Use transactions for multi-step state changes.
- Add constraints for important invariants.
- Review queries for organisation isolation.
- Add indexes based on expected access patterns.
- Use connection pooling with bounded connections.
- Test migrations against a real PostgreSQL instance in CI where practical.

### 6.3 Database migrations

Store migrations as paired, ordered `.up.sql` and `.down.sql` files under `packages/backend/migrations/versions/`. Apply them with the Psycopg-based runner using `make db-migrate` or `uv run python -m app.db.migrations upgrade`.

Every schema change must include a migration.

Requirements:

- Keep migration contents in SQL files rather than Python revision scripts.
- Give every upgrade migration a matching downgrade file.
- The runner tracks applied versions and checksums in `schema_migrations`, applies each file transactionally, and rejects modified applied migrations.
- Migrations must be repeatable in a clean environment.
- Migrations must be reviewed.
- Destructive changes must be treated carefully.
- Production deployment must apply migrations in a controlled step.
- Application rollback and schema rollback assumptions must be documented.

### 6.4 Database access rules

Every tenant-owned resource query must enforce the appropriate organisation scope.

Do not depend on the frontend to provide a trustworthy organisation identifier.

Consider PostgreSQL row-level security as an additional defence only if its session and transaction semantics are implemented and tested correctly. It must not substitute for application-level authorisation.

## 7. Background Processing

### 7.1 Celery

Use Celery for asynchronous analysis tasks.

Responsibilities:

- Dispatch analysis jobs.
- Execute descriptive statistics.
- Record progress where meaningful.
- Handle transient failures.
- Retry eligible tasks with bounded backoff.
- Persist results.
- Record final job states.

Tasks should be small enough to test independently and should not rely on in-memory API state.

### 7.2 Redis

Use Redis as the Celery broker in the initial implementation.

Configure:

- Connection timeouts.
- Authentication and network restrictions in deployed environments.
- Task time limits.
- Retry policies.
- Queue monitoring.
- Appropriate persistence or managed-service durability settings.

Redis is not the source of truth for application job records. PostgreSQL remains authoritative for job state and results.

### 7.3 Idempotency and recovery

Design tasks to tolerate repeated delivery.

The implementation should account for:

- Duplicate task messages.
- Worker restarts.
- Transient database failures.
- Result persistence failures.
- Jobs that exceed their time limits.
- Tasks that finish after their lease or ownership period expires.

Use explicit job-state transitions and concurrency controls to prevent conflicting workers from corrupting the same logical job.

## 8. Scientific Computing and EEG Data

### 8.1 NumPy

Use NumPy for numerical operations and descriptive statistics.

The implementation should document:

- Expected array dimensions.
- Channel ordering.
- Sampling-frequency units.
- Signal measurement units.
- Standard deviation convention.
- Handling of invalid values.
- Numerical precision expectations.

Avoid copying large arrays unnecessarily.

### 8.2 Initial recording format

For the simplest MVP, use a documented format based on:

- A NumPy `.npy` array.
- A JSON manifest containing the sampling frequency, channel names, units, and relevant provenance.

The manifest must be validated against the actual recording.

The implementation must explicitly document array orientation, for example `(channels, samples)`, and enforce it consistently.

Do not use Python pickle files for untrusted uploads.

If EDF support becomes a core requirement, introduce a maintained EEG-compatible parser as a separate, tested feature.

### 8.3 Resource limits

Set configurable limits for:

- Maximum upload size.
- Maximum channel count.
- Maximum recording duration.
- Maximum sample count.
- Maximum analysis execution time.
- Maximum visualisation interval.
- Maximum API response size.

Reject inputs that exceed safe limits before performing expensive processing.

## 9. Object Storage

### Google Cloud Storage

Use a private GCS bucket for source recordings and any large processing artefacts.

Requirements:

- Block public access.
- Apply least-privilege IAM.
- Encrypt data at rest.
- Use TLS for network access.
- Validate upload authorisation.
- Keep bucket identifiers and credentials in configuration or managed identity.
- Use short-lived signed URLs only when required.
- Avoid logging signed URLs.
- Define retention and deletion behaviour.
- Validate checksums and object metadata where appropriate.

Keep object references in PostgreSQL rather than embedding large file contents in relational rows.

Database and object-storage operations are not automatically atomic. Implement a clear upload-finalisation and failure-recovery workflow.

## 10. Cloud Infrastructure

### 10.1 Google Cloud Platform

GCP is the recommended deployment target.

A practical production-oriented architecture may use:

- Cloud Run for the API.
- A separate worker deployment for background processing.
- Cloud SQL for PostgreSQL.
- Cloud Storage for recordings.
- Secret Manager for secrets.
- Cloud Logging and Cloud Monitoring for operations.
- A managed Redis-compatible service or another documented broker deployment.

Verify that the selected queue service and worker hosting model support the required Celery behaviour before committing to a particular managed service.

If the chosen managed Redis service does not meet worker connectivity or delivery requirements, document and use an alternative supported deployment.

### 10.2 Development and production parity

Use Docker Compose for local development.

Local services should approximate the deployed architecture:

- Frontend.
- API.
- PostgreSQL.
- Redis.
- Worker.

Object storage can use a local emulator during development if it provides the required behaviour. Keep the storage interface independent of the emulator.

Production configuration must not rely on development defaults.

## 11. Infrastructure as Code

Use Terraform for cloud infrastructure.

Manage infrastructure resources such as:

- Application service configuration.
- Database instances.
- Object-storage buckets.
- Service accounts and IAM bindings.
- Secrets references.
- Logging and monitoring configuration.
- Networking and access controls.
- Required environment configuration.

Requirements:

- Do not commit Terraform state files or secrets.
- Use remote state for shared deployments.
- Enable state locking where supported.
- Review plans before applying changes.
- Separate environments where practical.
- Avoid granting broad project-level permissions when narrower permissions suffice.
- Document resource naming and ownership.

For the MVP, Terraform can be introduced after the application works locally, but infrastructure should be reproducible before claiming a production deployment.

## 12. Testing and Quality

### Frontend

Use:

- Vitest.
- React Testing Library.
- Playwright.
- TypeScript compiler checks.
- ESLint.

Test user-visible behaviour rather than implementation details.

### Backend

Use:

- pytest.
- HTTPX.
- Real PostgreSQL integration tests.
- Controlled worker and queue tests.
- Ruff.
- mypy.

Mock external services in unit tests where appropriate, but use real PostgreSQL for tests involving SQL semantics, constraints, transactions, or migrations.

### Scientific processing

Test:

- Known arrays with known expected statistics.
- Empty and malformed input.
- Invalid metadata.
- Non-finite values.
- Large input boundaries.
- Correct channel ordering.
- Numerical edge cases.
- Reproducibility for fixed input and parameters.

### End-to-end

Use Playwright for critical workflows:

1. Authenticate.
2. Create a study.
3. Upload a recording.
4. Inspect dataset metadata.
5. Explore a signal interval.
6. Submit an analysis.
7. Observe job completion.
8. Inspect the result.

## 13. CI/CD

Use GitHub Actions.

The CI pipeline should run:

- Frontend dependency installation.
- Frontend linting.
- TypeScript checks.
- Frontend unit tests.
- Backend dependency installation.
- Python linting and formatting checks.
- Python type checking.
- Backend unit tests.
- Database integration tests.
- Migration validation.
- Build verification.
- Relevant security and dependency checks.

The deployment pipeline should:

1. Build immutable artefacts.
2. Run automated checks.
3. Deploy to a non-production environment where configured.
4. Apply migrations in a controlled manner.
5. Run smoke tests.
6. Promote to production only when the configured checks succeed.

Never deploy production changes directly from an unreviewed local working tree.

## 14. Dependency Management

### Python

Use `uv` for Python project and dependency management.

Maintain a committed lockfile and a documented Python version.

Separate runtime dependencies from development and testing dependencies.

### TypeScript

Use `pnpm` for JavaScript dependency management.

Commit the lockfile and use frozen-lockfile installation in CI.

### Git hooks

Use Husky to install the repository's Git hooks through the root `prepare` script. The `pre-commit` hook runs `lint-staged`, which applies Ruff lint fixes and formatting to staged Python files and ESLint plus Prettier to staged frontend JavaScript/TypeScript files. Install dependencies with `pnpm install` to configure hooks; CI remains responsible for full tests, type checks, and builds.

### Version policy

- Pin runtime versions deliberately.
- Avoid floating production image tags.
- Review security advisories.
- Upgrade dependencies through tested pull requests.
- Avoid adding multiple libraries for the same responsibility.
- Document important version constraints.

## 15. Repository Organisation

A suggested repository structure:

```text
eeg-lab/
├── packages/
│   ├── backend/
│   │   ├── app/
│   │   │   ├── api/
│   │   │   ├── core/
│   │   │   ├── db/
│   │   │   ├── models/
│   │   │   ├── repositories/
│   │   │   ├── schemas/
│   │   │   ├── services/
│   │   │   ├── workers/
│   │   │   └── main.py
│   │   ├── migrations/
│   │   │   └── versions/
│   │   │       ├── 0001_initial_schema.up.sql
│   │   │       ├── 0001_initial_schema.down.sql
│   │   │       ├── 0002_add_upload_sessions_manifest.up.sql
│   │   │       └── 0002_add_upload_sessions_manifest.down.sql
│   │   ├── tests/
│   │   └── pyproject.toml
│   ├── frontend/
│   │   ├── src/
│   │   ├── public/
│   │   ├── tests/
│   │   └── package.json
│   └── api-client/          proposed shared behaviour contract
├── infra/
│   └── terraform/
├── scripts/
├── docs/
├── docker-compose.yml
├── Makefile
├── .github/
│   └── workflows/
└── README.md
```

This is a proposed structure, not a requirement to create every directory immediately.

Prefer a small, coherent implementation over empty placeholder modules.

## 16. Configuration and Secrets

Use environment variables or a documented configuration mechanism.

Configuration should include:

- Environment name.
- API base URL.
- Database connection settings.
- Redis broker URL.
- Object-storage configuration.
- Authentication provider settings.
- Upload and processing limits.
- Logging level.
- CORS origins.

Requirements:

- Validate required configuration at startup.
- Fail clearly when mandatory configuration is missing.
- Never commit real credentials.
- Provide an `.env.example` with placeholders only.
- Keep local development secrets out of version control.
- Use managed identities and a secrets manager in deployed environments where supported.

## 17. Deferred Technologies

Do not introduce these technologies in the initial MVP unless a demonstrated requirement justifies them:

- Kubernetes.
- A service mesh.
- Kafka.
- Multiple databases.
- A heavyweight ORM.
- A second backend language.
- Distributed tracing infrastructure beyond what the initial operational needs justify.
- GPU-based computation.
- A dedicated feature store.
- Large-scale machine-learning orchestration.
- Real-time EEG streaming infrastructure.

These may be relevant to a larger research platform, but they increase implementation and operational cost before the core product is proven.

## 18. Technology Decision Record Requirements

Record major architectural decisions in `docs/decisions/`.

Each decision should include:

- Context.
- Decision.
- Alternatives considered.
- Reasons for the choice.
- Trade-offs.
- Consequences.
- Conditions that would justify revisiting the decision.

Create decision records for at least:

1. Explicit SQL versus ORM.
2. Recording file format.
3. Background job architecture.
4. Authentication approach.
5. Object-storage strategy.
6. Deployment and queue topology.

## 19. Definition of Done

The technology stack is correctly implemented when:

1. Frontend and backend have independent build and test commands.
2. Type checking and linting are enforced.
3. PostgreSQL schema changes use migrations.
4. Database queries are parameterised.
5. Background processing is separated from API request handling.
6. Large scientific files are stored outside PostgreSQL.
7. Configuration and secrets are handled safely.
8. Local development is reproducible.
9. CI validates the critical application workflows.
10. Cloud deployment can be reproduced through documented infrastructure configuration.
11. Every selected technology has a clear responsibility.
12. The implementation does not introduce deferred technologies without a documented reason.

### 4.5 Frontend coding conventions

- Use arrow functions only for all function declarations (components, hooks, utilities, handlers, and callbacks). Do not use `function` declarations.
- Prefer `const` for all variables; avoid `var`.
- Use named exports for utilities and components; use default exports for page and layout components where required by the framework.
- Keep components small and focused. Extract reusable logic into custom hooks or utilities.
- Use TypeScript for all source files. Avoid `any` unless absolutely necessary and documented.
- Place tests in `__tests__` directories colocated with the code they test (e.g., `src/components/__tests__/button.test.tsx`).
