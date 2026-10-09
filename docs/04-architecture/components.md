## 4. Component Responsibilities

### 4.1 Frontend application

Technology: Next.js, React, TypeScript.

Responsibilities:

- Authentication experience.
- Organisation selection.
- Study management.
- Dataset upload.
- Metadata presentation.
- Signal exploration.
- Analysis submission.
- Job status monitoring.
- Result presentation.
- Accessible error and loading states.

The frontend is not trusted to enforce permissions or validate scientific data conclusively.

Browser code talks to the Python API only through the tRPC Backend-for-Frontend that runs inside the Next.js server (see `08-tech-stack.md` 4.5). The BFF reads the HttpOnly session cookie, resolves the active organisation on the server, and forwards authenticated REST requests; it never weakens the checks performed by the API.

Client-side validation improves usability, but the API must repeat all security-relevant checks.

### 4.2 API service

Technology: FastAPI and Pydantic.

Responsibilities:

- Authenticate requests.
- Resolve the current user.
- Verify organisation membership and permissions.
- Validate API input.
- Apply business rules.
- Manage relational metadata.
- Coordinate file uploads and finalisation.
- Create analysis jobs.
- Return bounded signal data.
- Retrieve analysis results.
- Emit structured logs.
- Expose health and readiness endpoints.

The API should remain stateless between requests, apart from appropriately managed external session mechanisms.

### 4.3 Application services

Application services implement complete business operations, such as:

- Creating a study.
- Finalising a dataset upload.
- Validating a recording.
- Submitting an analysis.
- Claiming or completing a job.
- Persisting an analysis result.

Services coordinate repositories and external integrations.

They should not contain presentation-specific logic or rely on implicit global state.

### 4.4 SQL repositories

Repositories encapsulate explicit SQL queries and transaction boundaries.

Responsibilities:

- Reading and writing domain records.
- Applying organisation-scoped filters.
- Handling concurrency.
- Enforcing expected state transitions.
- Retrieving paginated results.
- Mapping database rows to application structures.

Use parameterised SQL for all user-controlled values.

Repositories must not accept an organisation identifier from the client as proof of authorisation. They receive an organisation context already verified by the application layer.

### 4.5 Scientific processing module

Technology: Python and NumPy.

Responsibilities:

- Load or stream supported recording data within configured limits.
- Validate numerical arrays.
- Compute descriptive statistics.
- Return typed, serialisable results.
- Apply documented numerical conventions.
- Avoid modifying source data.

Keep numerical functions as deterministic and independently testable as practical.

### 4.6 Background worker

Technology: Celery.

Responsibilities:

- Execute queued analyses.
- Load authorised job inputs.
- Apply resource and time limits.
- Persist successful results.
- Record safe failure information.
- Retry eligible transient errors.
- Recover from interrupted execution.

A worker must verify the current job state before committing results. Receiving a duplicate message must not automatically mean the job can run again.

### 4.7 PostgreSQL

PostgreSQL is the source of truth for application metadata and job state.

It stores relational records, ownership relationships, lifecycle state, audit events, and analysis results.

### 4.8 Redis

Redis transports background task messages.

It is not the authoritative source of analysis job state. The application must be able to inspect job status from PostgreSQL even when Redis is temporarily unavailable.

### 4.9 Object storage

Private object storage holds EEG recordings and any large processing artefacts.

Store object keys and metadata in PostgreSQL. Do not store public URLs as a substitute for access control.

