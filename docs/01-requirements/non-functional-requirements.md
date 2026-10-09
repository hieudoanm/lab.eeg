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
