# EEG Lab — Requirements

## 1. Purpose

This document defines the functional requirements, non-functional requirements, technical constraints, and acceptance criteria for EEG Lab.

All implementation decisions must align with these requirements and the project's `01-overview.md`, `04-features.md`, and `05-architecture.md`.

Requirements should be implemented incrementally according to `11-implementation-plan.md`. Do not attempt to implement the entire platform in a single pass.

## 2. Scope

The MVP must provide:

- User authentication and organisation membership.
- Study management.
- EEG dataset upload, validation, and provenance tracking.
- Interactive scientific signal visualisation.
- Asynchronous descriptive analysis.
- Analysis job monitoring and result inspection.
- Role-based access control and tenant isolation.
- Automated tests and continuous integration.
- Reproducible local development and documented cloud deployment.

Advanced machine learning, real-time acquisition, clinical interpretation, and commercial billing are outside the initial scope.

## 3. Functional Requirements

### 3.1 Authentication and Identity

**FR-001 — User authentication**

The application must authenticate users before granting access to private resources.

**FR-002 — Current user**

The API must expose an endpoint that returns the authenticated user's identity and accessible organisations.

**FR-003 — Organisation membership**

A user may belong to multiple organisations. Membership must be represented explicitly in the database.

**FR-004 — Organisation selection**

The frontend must allow users to select an organisation they belong to.

**FR-005 — Server-side verification**

The API must verify organisation membership for every organisation-scoped request. Client-supplied organisation identifiers must never be trusted without verification.

**FR-006 — Development authentication**

Local development may use a seeded account or development-only authentication mechanism. This mechanism must be disabled by default in production and must fail closed when production configuration is invalid.

**FR-007 — Session handling**

Authentication failures and expired sessions must be handled safely. The frontend must not enter an infinite retry or redirect loop.

### 3.2 Roles and Permissions

The system must support three roles:

| Capability                  | Owner | Researcher | Viewer |
| --------------------------- | ----- | ---------- | ------ |
| View permitted studies      | Yes   | Yes        | Yes    |
| Create studies              | Yes   | Yes        | No     |
| Edit studies                | Yes   | Yes        | No     |
| Archive studies             | Yes   | Yes        | No     |
| Upload datasets             | Yes   | Yes        | No     |
| View datasets and results   | Yes   | Yes        | Yes    |
| Submit analyses             | Yes   | Yes        | No     |
| Manage organisation members | Yes   | No         | No     |

Additional rules:

- **FR-008:** The API must enforce permissions independently of frontend visibility.
- **FR-009:** Role changes must be validated and recorded.
- **FR-010:** Users must not be able to grant themselves additional permissions.
- **FR-011:** A user must not be able to remove or demote the last active organisation owner without a valid ownership-transfer process.
- **FR-012:** The application must return a consistent access-denied response without leaking sensitive resource information.

### 3.3 Study Management

**FR-013 — Create studies**

Authorised users must be able to create a study with a required name and optional description.

**FR-014 — List studies**

Users must be able to list studies available within their active organisation.

The list must support:

- Pagination.
- Deterministic sorting.
- Search by study name.
- Study status.
- Creation and update timestamps.

**FR-015 — View study details**

Users must be able to view study metadata, associated datasets, and associated analyses.

**FR-016 — Update studies**

Authorised users must be able to update supported study fields.

**FR-017 — Archive studies**

Studies should be archived rather than permanently deleted by default. Archived studies must retain their historical datasets and analysis records.

**FR-018 — Validate study input**

The server must validate required fields, field lengths, and supported status transitions.

**FR-019 — Handle empty states**

The frontend must explain when no studies exist and provide an appropriate create action to authorised users.

### 3.4 Dataset Upload and Validation

**FR-020 — Upload supported EEG data**

Researchers must be able to upload a supported EEG recording through the web application.

**FR-021 — Supported format**

The MVP must implement and document at least one supported signal-file format. A practical initial choice is NumPy `.npy` data accompanied by a JSON metadata manifest. Additional formats may be added when needed.

If EDF or another specialist EEG format is selected instead, document the parser and its dependencies in `08-tech-stack.md`.

**FR-022 — Upload metadata**

Dataset metadata must include, where applicable:

- Dataset name.
- Source name and source URL.
- Recording identifier.
- Sampling frequency.
- Channel names.
- Number of channels and samples.
- Units.
- File checksum.
- Creation timestamp.
- Validation status.

**FR-023 — Validate uploads**

The backend must validate:

- File size.
- File extension and actual content.
- Required metadata.
- Sampling frequency and dimensions.
- Channel metadata consistency.
- Numeric values and finite-value constraints.
- Maximum duration, channel count, and sample count.
- Parser resource limits.

**FR-024 — Validation status**

A dataset must have a documented lifecycle:

`pending_upload` → `validating` → `ready`

Alternative terminal states are `rejected` and `failed`.

Only validated datasets in the `ready` state may be analysed.

**FR-025 — Validation reports**

The application must store a structured validation report and present actionable validation errors to users.

**FR-026 — Private storage**

Raw recordings must be stored outside the relational database in private object storage or the documented local storage adapter.

**FR-027 — Integrity**

The system must calculate and persist a checksum for uploaded files to support integrity verification.

**FR-028 — Immutable source data**

A source recording must not be silently overwritten. Replacing a recording must create a new dataset version or a new dataset record according to the documented versioning strategy.

**FR-029 — Upload completion**

If uploads use presigned URLs, the backend must verify that the expected object exists and passes integrity checks before accepting the upload as complete.

### 3.5 Signal Exploration

**FR-030 — Dataset details**

Users must be able to inspect a dataset's metadata, provenance, validation state, and available channels.

**FR-031 — Signal window**

Users must be able to select a bounded time interval for visualisation.

**FR-032 — Channel selection**

Users must be able to select one or more channels within configured limits.

**FR-033 — Interactive chart**

The frontend must render the selected signal using Plotly.js or an equivalent documented scientific visualisation library.

**FR-034 — Chart metadata**

The visualisation must include:

- Time axis.
- Signal amplitude axis.
- Channel labels.
- Sampling frequency.
- Measurement units when known.
- Indication of display downsampling, when applicable.

**FR-035 — Bounded responses**

The API must enforce maximum time windows, channel counts, and returned point counts. A full recording must never be returned by default.

**FR-036 — Analytical integrity**

Display downsampling must not alter the source recording or become an implicit input to scientific analysis.

**FR-037 — Accessible alternative**

Charts must have meaningful titles and descriptions, with a textual or tabular alternative for users who cannot interpret the visualisation.

**FR-038 — Research disclaimer**

The application must communicate that its visualisations and descriptive statistics are for research exploration, not clinical diagnosis.

### 3.6 Asynchronous Analysis

**FR-039 — Submit analysis**

Authorised users must be able to submit a descriptive analysis for a validated dataset.

**FR-040 — Background execution**

Analysis computation must execute outside the synchronous API request handler.

**FR-041 — Initial analysis**

The MVP must calculate at least the following metrics for each channel:

- Mean.
- Standard deviation.
- Minimum.
- Maximum.
- Peak-to-peak range.
- Number of valid samples.

The implementation must document the standard deviation convention and how invalid or non-finite values are handled.

**FR-042 — Job states**

Analysis jobs must support:

`queued`, `running`, `succeeded`, `failed`, and `cancelled`.

**FR-043 — State transitions**

The worker must validate job-state transitions. A completed job must not return to `running` or become completed twice without a documented recovery mechanism.

**FR-044 — Idempotency**

Repeated requests with the same dataset version, analysis type, algorithm version, and idempotency key must not create duplicate work or conflicting results.

**FR-045 — Retry policy**

Transient failures may be retried using bounded attempts and backoff. Invalid parameters and permanently malformed input must not be retried indefinitely.

**FR-046 — Job monitoring**

The frontend must display job status, creation time, execution timestamps, and safe error information.

**FR-047 — Result persistence**

Successful jobs must persist structured results and their associated provenance.

**FR-048 — Reproducibility**

Each result must record the algorithm version, parameters, dataset version, and relevant execution metadata.

**FR-049 — Failure recovery**

The system must recover or safely classify jobs that remain in the `running` state after a worker crash.

**FR-050 — Result access**

Analysis results must be accessible only to authorised users belonging to the relevant organisation.

### 3.7 API and Data Access

**FR-051 — Versioned REST API**

Application endpoints must be versioned under `/api/v1`.

**FR-052 — Pydantic validation**

Request and response payloads must use explicit Pydantic models.

**FR-053 — Consistent errors**

The API must return a consistent error envelope with a stable error code, safe message, and request identifier.

**FR-054 — Parameterised SQL**

All SQL values must be passed as query parameters. User input must never be concatenated into SQL statements.

**FR-055 — Pagination**

List endpoints must support pagination with a configurable default page size and a strict maximum.

**FR-056 — Safe sorting and filtering**

Sort fields and filter operations must use predefined allowlists. User input must not become an arbitrary SQL identifier or expression.

**FR-057 — Streaming support**

After the core workflow works, implement at least one appropriate large-result endpoint using streaming responses or server-side cursors. Database connections and cursors must be closed on completion, errors, and client cancellation.

**FR-058 — Request tracing**

The API must attach a request identifier to responses and structured logs.

**FR-059 — API documentation**

The application must expose generated OpenAPI documentation for the implemented API.

### 3.8 Multi-Tenancy

**FR-060 — Tenant isolation**

Every organisation-owned resource must be isolated from other organisations.

**FR-061 — Database isolation strategy**

The architecture must explicitly document and implement one tenant-isolation strategy.

The preferred target-role demonstration is a shared control plane with a separate PostgreSQL schema per organisation. If this creates excessive delivery risk, implement a thoroughly tested organisation-scoped data model first and document schema-per-tenant as a subsequent milestone.

**FR-062 — Tenant-aware queries**

Every database operation must use a verified tenant context. Tenant context must not be derived solely from a client-controlled value.

**FR-063 — Object access**

Object-storage access must verify the caller's permissions before issuing a download URL or returning file contents.

**FR-064 — Cross-tenant testing**

Automated tests must verify isolation for studies, datasets, uploads, jobs, results, and downloadable objects.

**FR-065 — Tenant-safe background processing**

Jobs must retain sufficient trusted organisation context for the worker to process data in the correct tenant.

### 3.9 Audit Events

**FR-066 — Security-relevant events**

The system must record important events, including study creation and archiving, membership changes, dataset uploads, validation outcomes, and analysis submission or retry.

**FR-067 — Audit fields**

An audit event should include actor, organisation, action, resource type, resource identifier, timestamp, and request identifier when available.

**FR-068 — Safe audit data**

Audit records must not contain credentials, access tokens, signed URLs, or unnecessary raw signal data.

### 3.10 Operations

**FR-069 — Health check**

The API must expose `/healthz` to report process health.

**FR-070 — Readiness check**

The API must expose `/readyz` to report whether required dependencies are available, without exposing secrets or connection strings.

**FR-071 — Structured logging**

Application and worker logs must include useful context, severity, and request or job identifiers.

**FR-072 — Local development**

The application must be runnable locally using documented commands and a reproducible dependency setup.

**FR-073 — Continuous integration**

CI must execute relevant formatting, linting, type checks, tests, and build validation.

The repository must also install Husky Git hooks that run lint-staged on staged Python and frontend source files before commit. These fast local checks supplement, and do not replace, CI.

**FR-074 — Infrastructure as code**

Cloud infrastructure must be provisionable using Terraform.

**FR-075 — Deployment safety**

Deployment and migration procedures must be documented. Destructive changes must not be applied automatically from unreviewed pull requests.

---

## 4. Non-Functional Requirements

### 4.1 Security

- NFR-001: Authentication and authorisation must be enforced on the server.
- NFR-002: Access must be denied by default.
- NFR-003: All database queries must use parameterised SQL.
- NFR-004: Secrets must never be committed to source control or exposed to frontend code.
- NFR-005: Production traffic must use HTTPS.
- NFR-006: Object storage must be private by default.
- NFR-007: File uploads must have strict size and parsing limits.
- NFR-008: Development authentication must not be enabled accidentally in production.
- NFR-009: Logs and error responses must not expose credentials, raw data, or internal stack traces.
- NFR-010: Security-critical behaviours must have automated tests.

### 4.2 Reliability

- NFR-011: Background jobs must tolerate repeated delivery.
- NFR-012: Database transactions must protect consistency-critical operations.
- NFR-013: Failed jobs must have clear states and bounded retry policies.
- NFR-014: The application must distinguish transient failures from invalid input.
- NFR-015: Database migrations must be versioned and repeatable.

### 4.3 Performance

- NFR-016: Expensive scientific computation must run outside API request handlers.
- NFR-017: Signal endpoints must enforce strict response-size limits.
- NFR-018: List endpoints must enforce pagination limits.
- NFR-019: Large files must use object storage rather than large relational database fields.
- NFR-020: Scientific charts must avoid rendering unnecessarily large arrays in the browser.
- NFR-021: Performance decisions must be supported by measurements where possible rather than arbitrary optimisation.

### 4.4 Maintainability

- NFR-022: TypeScript must use strict type checking.
- NFR-023: Python code must use type annotations for application interfaces.
- NFR-024: Database changes must use versioned migrations.
- NFR-025: Frontend, API, persistence, worker, and scientific computation must have clear boundaries.
- NFR-026: Public API contracts and operational procedures must be documented.
- NFR-027: Avoid unnecessary abstractions, dependencies, and distributed services.

### 4.5 Accessibility and Usability

- NFR-028: Primary workflows must be keyboard accessible.
- NFR-029: Interactive controls must have semantic labels.
- NFR-030: Focus states and sufficient colour contrast must be provided.
- NFR-031: Status must not be communicated by colour alone.
- NFR-032: Charts must provide an accessible textual alternative.
- NFR-033: Forms must provide actionable validation errors.
- NFR-034: Loading, empty, success, and error states must be implemented consistently.

### 4.6 Observability

- NFR-035: Requests and jobs must be traceable through stable identifiers.
- NFR-036: Logs must be structured and suitable for operational troubleshooting.
- NFR-037: Health and readiness checks must distinguish process health from dependency availability.
- NFR-038: Job failures and queue delays must be observable.
- NFR-039: Monitoring must avoid unnecessary collection of personal or scientific data.

### 4.7 Cost and Portability

- NFR-040: Local development must not require paid cloud services.
- NFR-041: Cloud deployment must be opt-in.
- NFR-042: Infrastructure documentation must explain likely costs and cleanup procedures.
- NFR-043: Cloud-specific operations should be isolated behind small interfaces when this materially improves local testing or maintainability.

---

## 5. Technical Constraints

1. Use React and TypeScript for the web application.
2. Use Python, FastAPI, and Pydantic for the API.
3. Use PostgreSQL as the source of truth for application metadata and job state.
4. Use asynchronous PostgreSQL access through `asyncpg` or a justified equivalent.
5. Use explicit SQL rather than an ORM.
6. Use a separate worker process for asynchronous analysis.
7. Store raw recordings and large outputs in object storage.
8. Use Plotly.js or a justified equivalent for signal visualisation.
9. Use Terraform for infrastructure.
10. Use GitHub Actions for continuous integration.
11. Use small synthetic fixtures in the repository rather than large raw scientific datasets.
12. Follow existing repository conventions where they are sound; do not restructure a working repository unnecessarily.

## 6. Acceptance Criteria

The MVP must satisfy the following end-to-end acceptance conditions.

### AC-001 — Study creation

Given an authenticated researcher with valid organisation membership, when they create a study with a valid name, the study is persisted and appears in the study list.

### AC-002 — Invalid study input

Given an invalid study payload, when the API receives it, the API returns a documented validation error and does not create a partial study.

### AC-003 — Dataset validation

Given a supported EEG file and valid metadata, when the upload and validation workflow completes, the dataset becomes `ready` and its metadata is available.

### AC-004 — Invalid dataset

Given an oversized, malformed, or unsupported file, when validation runs, the dataset is rejected or marked failed with an actionable reason. It must not be available for analysis.

### AC-005 — Signal exploration

Given a ready dataset, when an authorised user requests a bounded time window, the API returns the requested channels and signal values within configured limits.

### AC-006 — Asynchronous analysis

Given a ready dataset and valid analysis parameters, when an authorised user submits an analysis, the API returns promptly with an accepted job resource, and the worker processes the job asynchronously.

### AC-007 — Analysis correctness

Given a known test signal, when descriptive statistics are computed, the results match independently verified expected values within the chosen numerical tolerance.

### AC-008 — Idempotency

Given the same dataset version and idempotency key, when a request is repeated, the system does not create conflicting duplicate results.

### AC-009 — Role enforcement

Given a viewer, when they attempt a write operation, the API rejects the operation even if the request is made directly without using the frontend.

### AC-010 — Tenant isolation

Given users in separate organisations, when one user attempts to access another organisation's studies, datasets, jobs, results, or files, the request is denied without leaking protected information.

### AC-011 — Worker recovery

Given a worker failure during processing, when recovery or retry occurs, the job eventually reaches a valid state without creating duplicate results.

### AC-012 — Local reproducibility

Given a clean checkout and the documented prerequisites, when a developer follows the setup guide, the application and required local dependencies start successfully.

### AC-013 — CI quality gates

Given a pull request, CI runs the required automated checks and fails when a mandatory check fails.

### AC-014 — Cloud reproducibility

Given valid cloud credentials and the required permissions, an operator can provision and deploy the documented cloud environment using Terraform and the deployment instructions.

---

## 7. Assumptions and Open Decisions

The following decisions should be resolved early and documented in the relevant project files:

- Which authentication provider or local development mechanism will be used.
- Whether the first signal format will be NumPy plus a JSON manifest or EDF.
- Which durable queue implementation best fits the local and cloud environments.
- Whether schema-per-tenant isolation can be delivered safely within the initial scope.
- Which object-storage adapter will be used for local development.
- Which deployment configuration is appropriate for the available cloud budget.

Codex should choose the simplest secure option when a decision is not blocking, document the rationale, and avoid introducing unnecessary complexity.

## 8. Requirement Management

- Keep requirement identifiers stable when updating this document.
- Update affected documentation when scope or behaviour changes.
- Link critical requirements to tests where practical.
- Do not mark a requirement complete merely because a placeholder or interface exists.
- Treat tenant isolation, safe file handling, parameterised SQL, and secret protection as release-blocking requirements.
- Record deferred requirements explicitly rather than silently omitting them.
