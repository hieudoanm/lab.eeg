# EEG Lab — Implementation Plan

## 1. Purpose

This document defines the phased implementation plan for EEG Lab, an independent neuroscience research SaaS portfolio project.

The goal is to deliver a working, deployed application that demonstrates:

- Full-stack TypeScript and Python development.
- Secure multi-tenant application design.
- EEG dataset ingestion and visualisation.
- Scientific computing and reproducible analysis.
- Reliable asynchronous background processing.
- PostgreSQL data modelling and explicit SQL.
- Automated testing and CI/CD.
- Cloud deployment and infrastructure as code.
- Production-minded engineering decisions.

The project should be implemented incrementally. Each phase must produce a testable result before the next phase begins.

## 2. Delivery Principles

### Build a vertical slice first

The first objective is not to build every screen or establish the entire cloud infrastructure.

Build one complete workflow:

1. Open the application.
2. Authenticate or use an explicitly configured local development identity.
3. Create a study.
4. Register a synthetic EEG dataset.
5. Upload and validate the recording.
6. Submit an analysis.
7. Process it in a background worker.
8. Retrieve and visualise the result.

This workflow proves that the major components work together.

### Prefer working software over premature abstraction

- Implement only the features required by the current milestone.
- Avoid building a generic workflow engine.
- Avoid introducing a complex permission framework before the initial roles are understood.
- Avoid building a plugin system for a single analysis algorithm.
- Add abstractions when repeated behaviour or a clear architectural boundary justifies them.

### Keep scientific claims modest

The initial application performs descriptive EEG analysis. It does not diagnose conditions, interpret clinical outcomes, or claim that its outputs are validated for medical use.

### Protect the development feedback loop

- Run linting and type checking frequently.
- Add tests alongside features.
- Use small pull requests or commits.
- Keep the local development environment reproducible.
- Ensure a broken master branch is treated as a delivery problem.

## 3. Milestone Overview

| Phase | Milestone                 | Expected outcome                                  |
| ----- | ------------------------- | ------------------------------------------------- |
| 0     | Project foundation        | Repository, tooling, conventions, CI              |
| 1     | Backend foundation        | FastAPI, PostgreSQL, migrations, health checks    |
| 2     | Frontend foundation       | Next.js application shell and API client          |
| 3     | Identity and tenancy      | Organisation membership and role enforcement      |
| 4     | Studies and datasets      | Core CRUD workflows                               |
| 5     | EEG upload and validation | Secure ingestion of synthetic recordings          |
| 6     | Analysis pipeline         | Background jobs and persisted results             |
| 7     | Scientific visualisation  | Interactive EEG and analysis views                |
| 8     | Reliability and security  | Hardening, observability, failure recovery        |
| 9     | Portfolio polish          | Documentation, demo data, screenshots, case study |
| 10    | Cloud deployment          | GCP infrastructure and automated deployment       |

A milestone is complete only when its acceptance criteria are met.

## 4. Phase 0 — Project Foundation

### Objectives

Establish a reproducible development environment and repository structure.

### Tasks

- [x] Create the repository and initial README.
- [x] Add `AGENTS.md` with coding and repository conventions.
- [x] Add the 12 core project documentation files.
- [x] Define frontend and backend directory structures.
- [x] Configure Python dependency management.
- [x] Configure pnpm and Node.js versions.
- [x] Add formatting and linting tools.
- [x] Configure Husky and lint-staged for staged-source checks before commit.
- [x] Add `.editorconfig`.
- [x] Add environment variable examples.
- [x] Add a root-level Makefile or equivalent task runner.
- [x] Configure GitHub Actions for initial checks.
- [x] Add a `.gitignore` covering secrets, local data, build outputs, and generated scientific files.

### Suggested repository structure

```text
eeg-lab/
├── packages/
│   ├── frontend/
│   ├── backend/
│   └── shared-contracts/
├── workers/
│   └── analysis/
├── infrastructure/
│   └── terraform/
├── docs/
├── scripts/
├── tests/
├── .github/
│   └── workflows/
├── AGENTS.md
├── Makefile
├── README.md
└── .gitignore
```

The shared-contracts package is optional. Introduce it only if sharing generated API types provides a clear benefit.

### Acceptance criteria

- A new developer can follow the setup guide.
- The frontend and backend start locally.
- Formatting and linting commands run.
- Husky installs its pre-commit hook after dependency installation and checks staged source files.
- CI executes the initial checks.
- No credentials or real participant recordings are committed.
- Project documentation reflects the actual chosen tools.

## 5. Phase 1 — Backend Foundation

### Objectives

Create the application API and persistent database layer.

### Tasks

- [x] Create the FastAPI application.
- [x] Add configuration management.
- [x] Define development, test, and production settings.
- [x] Add structured logging.
- [x] Add request IDs.
- [x] Configure PostgreSQL connectivity using Psycopg 3.
- [x] Implement explicit parameterised SQL access patterns.
- [x] Add connection pooling.
- [x] Configure versioned SQL migrations and runner.
- [x] Create the initial schema.
- [x] Add `/health/live`.
- [x] Add `/health/ready`.
- [x] Add error response models.
- [x] Add pytest and HTTPX.
- [x] Add PostgreSQL integration tests.

### Database access conventions

- Use parameterised SQL for all untrusted values.
- Never interpolate user-controlled strings into SQL statements.
- Keep transaction boundaries explicit.
- Avoid holding database transactions open while performing long computations or remote object-storage operations.
- Do not share one database connection concurrently across unrelated asynchronous tasks.
- Keep SQL and persistence logic separated from route handlers.

### Acceptance criteria

- The API starts locally.
- PostgreSQL connectivity is verified.
- Migrations can build a fresh database.
- Health endpoints behave correctly.
- A database integration test passes against real PostgreSQL.
- An intentionally failing dependency causes readiness to fail appropriately.

## 6. Phase 2 — Frontend Foundation

### Objectives

Create the initial user interface and reliable communication with the API.

### Tasks

- [x] Create the Next.js application with TypeScript.
- [x] Configure Tailwind CSS.
- [x] Add the daisyUI component layer and map the EEG Lab design tokens to the `lab-light`/`lab-dark` daisyUI themes.
- [x] Establish design tokens and typography.
- [x] Create the application layout.
- [x] Create navigation and page shells.
- [x] Add loading, empty, error, and not-found states.
- [x] Create a typed API client.
- [x] Generate API types from OpenAPI if this is simpler than maintaining them manually.
- [x] Configure frontend unit tests.
- [x] Add accessible form components.
- [x] Add responsive layouts.
- [x] Add a basic Playwright smoke test.

### Initial pages

- Dashboard.
- Study list.
- Study detail.
- Dataset detail.
- Analysis detail.
- Settings or membership management, if required by the MVP.

### Acceptance criteria

- The application runs locally.
- Navigation works.
- Pages display meaningful loading and error states.
- API failures do not crash the entire application.
- Core forms provide accessible validation feedback.
- Frontend linting, type checking, and tests run in CI.

## 7. Phase 3 — Identity and Multi-Tenancy

### Objectives

Enforce secure organisation-scoped access before introducing real data workflows.

### Tasks

- [x] Choose the authentication approach.
- [x] Implement token or session validation.
- [x] Map authenticated identities to local user records.
- [x] Implement organisation membership lookup.
- [x] Implement role-based permission checks.
- [x] Add organisation-scoped query helpers.
- [x] Add tenant consistency constraints.
- [x] Add membership management for owners.
- [x] Add audit events for membership changes.
- [x] Test cross-organisation access attempts.

### Required roles

- `owner`
- `researcher`
- `viewer`

An operator role may be added later, but operational access must not automatically imply scientific data access.

### Security acceptance criteria

- Unauthenticated users cannot access protected endpoints.
- A viewer cannot submit analyses or modify datasets.
- A user cannot access another organisation's resources by changing a URL identifier.
- Referenced resources are checked for tenant consistency.
- Membership changes require appropriate permissions.
- Security-sensitive operations are auditable.

Do not move to real research data until these controls have been tested.

## 8. Phase 4 — Studies and Datasets

### Objectives

Deliver the core metadata-management workflow.

### Tasks

- [x] Implement study creation.
- [x] Implement study listing and detail retrieval.
- [x] Implement study updates.
- [x] Implement archive behaviour.
- [x] Implement dataset registration.
- [x] Implement dataset listing within a study.
- [x] Implement dataset detail retrieval.
- [x] Add pagination.
- [x] Add empty states.
- [x] Add form validation.
- [x] Add database and API tests.

### Acceptance criteria

- Researchers can create and manage studies.
- Viewers can read permitted study and dataset metadata.
- Dataset records belong to the correct organisation and study.
- Archived studies are handled consistently.
- Pagination is deterministic.
- Invalid resource relationships are rejected.

## 9. Phase 5 — EEG Upload and Validation

### Objectives

Support secure ingestion of synthetic EEG recordings.

### Initial format

Use NumPy `.npy` files with an explicit metadata manifest.

Define the array orientation as:

`(channels, samples)`

The upload workflow must not depend on assumptions about array shape that are undocumented or unvalidated.

### Tasks

- [x] Generate deterministic synthetic EEG recordings.
- [x] Create an upload-session API.
- [x] Configure private object storage.
- [x] Implement the chosen upload strategy.
- [x] Implement upload finalisation.
- [x] Verify object existence and expected size.
- [x] Calculate or verify checksums.
- [x] Validate file structure and dtype.
- [x] Validate dimensions and resource limits.
- [x] Validate channel metadata.
- [x] Validate sampling frequency.
- [x] Implement dataset lifecycle transitions.
- [x] Add failed-upload recovery.
- [x] Add cleanup for abandoned uploads.
- [x] Add malicious and malformed-file tests.
- [x] Add a dataset registration form.
- [x] Add an upload form with file selection, metadata, progress, and validation feedback.

### Implementation notes

- Validation runs synchronously during finalisation and returns `200 OK` with the
  terminal dataset state; the Phase 6 worker will move this to an asynchronous
  contract. See `docs/decisions/0003-upload-validation-strategy.md`.
- Uploads are backend-mediated in this phase (`upload_method: "backend"`); a
  direct-to-object-storage `signed_url` method is reserved for a later phase.
- The browser uploads recording bytes through a Next.js route handler
  (`/api/uploads/content`), which streams the request body to the authenticated
  API content endpoint using the session cookie's bearer token. The tRPC BFF
  carries only recording metadata, because its request bodies are capped at
  1 MB and large arrays must not travel through ordinary JSON endpoints.
- Abandoned sessions are expired lazily when a new session is requested for the
  same dataset, and on demand by `cleanup_abandoned_uploads`
  (`app.services.uploads`), exposed via `./scripts/cleanup_uploads.sh` and
  `make cleanup-uploads`. Cleanup claims non-terminal sessions past their TTL in
  bounded batches (`FOR UPDATE SKIP LOCKED`), expires them in a short
  transaction, and deletes their objects outside the transaction. Scheduling the
  command periodically is deferred to the reliability phase.
- Deterministic synthetic recordings are generated by
  `app.scientific.synthetic` (fixed seed, `(channels, samples)` float32), exposed
  via `./scripts/generate_recording.sh` and `make sample-recording`. The same
  module backs the unit fixtures used by the upload and validation tests.

### Acceptance criteria

- A researcher can upload a synthetic recording.
- The recording is stored privately.
- An upload does not make a dataset ready before validation.
- Invalid files are rejected with safe error messages.
- Metadata is persisted only after successful validation.
- Large arrays are not transferred through ordinary JSON endpoints.
- The application never loads untrusted NumPy data with pickle enabled.
- Failed uploads do not leave indefinitely stuck sessions.

## 10. Phase 6 — Analysis Pipeline

### Objectives

Implement durable asynchronous analysis execution.

### Initial analysis

`descriptive_statistics`

For each channel, calculate:

- Mean.
- Standard deviation.
- Minimum.
- Maximum.
- Peak-to-peak range.
- Valid sample count.

Document the treatment of missing or non-finite samples. Do not silently produce misleading statistics.

### Tasks

- [x] Define analysis request and parameter schemas.
- [x] Implement analysis submission.
- [x] Implement idempotency.
- [x] Persist job records and outbox events transactionally.
- [x] Implement outbox dispatch.
- [x] Configure Redis and Celery.
- [x] Implement the analysis worker.
- [x] Load recordings from private object storage.
- [x] Validate worker input independently.
- [x] Compute statistics using NumPy.
- [x] Persist validated results and provenance.
- [x] Implement job status polling.
- [x] Implement retry handling.
- [x] Implement stale-job recovery.
- [x] Implement cooperative cancellation if feasible.
- [x] Add worker integration tests.

### Required job states

- `queued`
- `running`
- `succeeded`
- `failed`
- `cancelled`

### Reliability acceptance criteria

- Submitting an analysis returns promptly.
- Job state persists across API restarts.
- Duplicate requests do not create duplicate logical jobs.
- A duplicate worker delivery does not corrupt results.
- Results are committed only after successful validation.
- A failed analysis does not return fabricated results.
- Worker crashes are observable and recoverable.
- Provenance includes dataset identity or checksum, algorithm version, parameters, and result schema version.

## 11. Phase 7 — Scientific Visualisation

### Objectives

Present recording metadata and numerical results clearly.

### Tasks

- [x] Integrate Plotly.js.
- [x] Create a channel-selection control.
- [x] Display a time-series view.
- [x] Calculate the time axis from sampling frequency.
- [x] Display channel metadata.
- [x] Display analysis summaries.
- [x] Display per-channel statistics.
- [x] Add analysis status indicators.
- [x] Add accessible alternatives to charts.
- [x] Add chart loading and failure states.
- [x] Test with recordings of different lengths.

### Visualisation requirements

- Use the actual sampling frequency when constructing time axes.
- Label axes and units where known.
- State when units are unknown.
- Avoid plotting millions of samples directly in the browser.
- Introduce server-side or worker-side downsampling when necessary.
- Do not describe descriptive statistics as clinical findings.
- Do not infer diagnoses or cognitive traits from the initial output.

### Acceptance criteria

- Users can inspect synthetic recordings.
- Channel selection works.
- Time axes reflect the supplied sampling frequency.
- Analysis values match backend results.
- Long recordings remain usable under documented limits.
- Charts have meaningful labels and accessible summaries.

## 12. Phase 8 — Reliability, Security, and Quality

### Objectives

Harden the complete vertical slice.

### Tasks

- [x] Review organisation isolation.
- [x] Review authentication and permissions.
- [x] Add request and resource limits.
- [x] Review upload security.
- [ ] Review signed URL handling.
- [x] Test transaction rollback behaviour.
- [x] Test duplicate event delivery.
- [x] Test stale-job recovery.
- [x] Add structured application and worker logs.
- [x] Add basic metrics.
- [x] Add correlation IDs.
- [x] Add database backup and recovery guidance.
- [x] Scan dependencies and container images.
- [x] Review secrets handling.
- [x] Add end-to-end tests.
- [x] Document known limitations.

### Acceptance criteria

- Critical security tests pass.
- Failures produce actionable logs without leaking secrets.
- Background jobs can recover from expected transient failures.
- Database and object-storage responsibilities are documented.
- The system has a tested recovery path for its initial deployment.
- The complete workflow passes automated end-to-end tests.

## 13. Phase 9 — Portfolio Polish

### Objectives

Make the project easy to evaluate by recruiters, engineers, and research teams.

### Tasks

- [x] Write a concise project README.
- [x] Add architecture and data-flow diagrams.
- [ ] Add screenshots of key workflows.
- [x] Add a short demonstration video or GIF if useful (`demo/eeg-lab-demo.mp4`).
- [ ] Publish an accessible demo where feasible.
- [x] Include synthetic example data.
- [x] Document setup and common troubleshooting steps.
- [x] Document key engineering decisions.
- [x] Add a threat model.
- [x] Add a scientific limitations statement.
- [x] Add test and CI status indicators.
- [x] Document infrastructure costs and teardown instructions.
- [x] Prepare a short portfolio case study.

### Portfolio narrative

Emphasise the engineering problems solved:

- Secure multi-tenant resource access.
- Reproducible scientific computation.
- Reliable asynchronous processing.
- Data integrity and provenance.
- Scientific visualisation.
- Infrastructure automation.
- Automated quality assurance.

Explain why decisions were made, what alternatives were considered, and which limitations remain.

Do not imply that EEG Lab is affiliated with or uses proprietary technology from BIOS Health or any other employer.

## 14. Phase 10 — Cloud Deployment

### Objectives

Deploy EEG Lab to Google Cloud Platform using infrastructure as code.

### Proposed services

- Cloud Run for the API.
- Cloud Run for the frontend if the selected Next.js deployment model is compatible.
- Cloud SQL for PostgreSQL.
- Private Cloud Storage buckets for recordings.
- A managed Redis-compatible service for the Celery broker.
- A separate worker deployment.
- Secret Manager.
- Cloud Logging and Monitoring.
- Terraform.
- GitHub Actions.

Verify service availability, regional support, authentication requirements, and cost before choosing exact managed-service configurations.

### Tasks

- [x] Create Terraform modules.
- [x] Define development and production environments.
- [x] Configure workload identity federation for GitHub Actions where supported.
- [x] Configure least-privilege service accounts.
- [x] Configure private object-storage access.
- [x] Configure database connectivity.
- [ ] Deploy the API.
- [ ] Deploy the worker.
- [ ] Deploy the frontend.
- [ ] Run database migrations safely.
- [ ] Configure health checks.
- [ ] Configure logs and alerts.
- [ ] Configure backup and retention policies.
- [ ] Run smoke tests against the deployed application.

> **Status:** the Terraform modules and Cloud Build pipeline are unvalidated
> scaffolding. They have not been applied against a live Google Cloud project,
> `terraform init`/`plan` and the Cloud Build pipeline have not been verified end
> to end, and no environment has been deployed. The checked configuration tasks
> above describe intended design until an environment is actually provisioned.

### Acceptance criteria

- Infrastructure can be provisioned reproducibly.
- Application credentials are not committed to source control.
- The worker can access only the required services.
- The database is not exposed unnecessarily to the public internet.
- Object storage is private by default.
- Deployment failures are visible.
- The application passes deployed smoke tests.
- Resource limits and estimated costs are documented.

## 15. Continuous Integration

Every pull request should run the checks relevant to the changed code.

### Frontend

- Formatting.
- Linting.
- Type checking.
- Unit tests.
- Production build.
- Selected component or browser tests.

### Backend

- Formatting.
- Linting.
- Type checking where configured.
- Unit tests.
- API tests.
- PostgreSQL integration tests.
- Migration checks.

### Worker

- Formatting and linting.
- Unit tests.
- Analysis correctness tests.
- Job lifecycle tests.
- Retry and duplicate-delivery tests.

### Infrastructure

- Terraform formatting.
- Terraform validation.
- Static security checks.
- Plan review for deployment changes.

Build and test containers using pinned or otherwise controlled dependency versions.

## 16. Suggested Weekly Execution Strategy

The project should be prioritised by dependency and risk, not by visual appeal.

| Priority | Work                                                        |
| -------- | ----------------------------------------------------------- |
| P0       | Repository, database, API, authentication, tenant isolation |
| P1       | Studies, datasets, upload validation                        |
| P2       | Analysis jobs, worker reliability, persisted results        |
| P3       | Visualisation and end-to-end workflow                       |
| P4       | Cloud deployment and operational safeguards                 |
| P5       | Portfolio polish and optional enhancements                  |

A small, working deployment is more valuable than a large unfinished feature set.

If time is limited, defer advanced dashboards, complex collaboration features, custom analysis builders, and elaborate visual effects.

## 17. Risk Register

| Risk                             | Impact                          | Mitigation                                                  |
| -------------------------------- | ------------------------------- | ----------------------------------------------------------- |
| Scope expansion                  | Delayed delivery                | Keep the first analysis deliberately small                  |
| Tenant isolation bug             | Data exposure                   | Centralised permission checks and integration tests         |
| Unsafe file parsing              | Security and reliability issues | Strict format validation and resource limits                |
| Duplicate job delivery           | Inconsistent results            | Idempotent workers and database state checks                |
| Lost queue messages              | Stuck jobs                      | Transactional outbox and recovery mechanisms                |
| Excessive cloud cost             | Unexpected expense              | Budgets, resource limits, alerts, and teardown instructions |
| Scientific misinterpretation     | Misleading claims               | Clear units, provenance, and limitations                    |
| Complex infrastructure too early | Slow development                | Prove the local vertical slice first                        |
| Inconsistent API types           | Integration defects             | Generate types from OpenAPI where practical                 |
| Insufficient test realism        | Hidden database bugs            | Use real PostgreSQL integration tests                       |

## 18. Definition of Project Completion

The MVP is complete when:

1. A user can access an authorised organisation.
2. A researcher can create a study.
3. A researcher can register and upload a synthetic EEG dataset.
4. The API validates the uploaded file.
5. The dataset becomes ready only after validation.
6. A researcher can submit a descriptive analysis.
7. A background worker executes the analysis.
8. Results and provenance are persisted.
9. The frontend displays the recording and analysis results.
10. Organisation boundaries are enforced and tested.
11. Automated checks pass in CI.
12. A documented deployment can be reproduced.
13. The README explains setup, architecture, testing, and limitations.

A feature is not complete merely because its primary happy path works. Validation, failure handling, permission checks, and observability must also be addressed.

## 19. Post-MVP Opportunities

Only consider these after the MVP is stable:

- EDF support through a carefully validated scientific file-reading library.
- Additional signal-processing operations.
- Analysis comparison and version history.
- Reproducible experiment configurations.
- Dataset export.
- Richer collaborative research workflows.
- Alternative scientific visualisations.
- More detailed operational dashboards.
- Additional neuroimaging formats.

Each extension should have a clear user need, documented acceptance criteria, and a test strategy before implementation.
