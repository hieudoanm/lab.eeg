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

