# EEG Lab — Deployment and Operations

## 1. Purpose

This document defines how to run, test, deploy, monitor, maintain, and safely tear down EEG Lab.

EEG Lab is an independent research-oriented neuroscience SaaS portfolio project built with:

- Next.js, React, and TypeScript.
- Python, FastAPI, and Pydantic.
- PostgreSQL and Psycopg 3.
- Celery and Redis.
- NumPy and Plotly.js.
- Private Google Cloud Storage.
- Google Cloud Platform.
- Terraform.
- GitHub Actions.

The deployment strategy prioritises reproducibility, security, reasonable operating costs, and a clear path from local development to a publicly accessible demonstration.

The exact cloud configuration must be verified against current provider capabilities and project requirements before implementation.

## 2. Deployment Principles

1. Keep local development straightforward.
2. Keep production credentials out of source control.
3. Separate frontend, API, and background-worker responsibilities.
4. Keep scientific recordings in private object storage.
5. Keep PostgreSQL inaccessible to unauthorised public clients.
6. Use least-privilege service accounts.
7. Manage infrastructure through Terraform.
8. Apply database migrations through a controlled deployment process.
9. Make deployments observable and recoverable.
10. Minimise always-on infrastructure for a portfolio project.
11. Document recurring costs and resource cleanup.
12. Never use real participant data in the public demonstration without an appropriate data-governance process.

## 3. Environment Strategy

Use three logical environments.

| Environment     | Purpose                        | Data                        |
| --------------- | ------------------------------ | --------------------------- |
| Local           | Daily development              | Synthetic data              |
| CI              | Automated testing              | Isolated synthetic fixtures |
| Production demo | Public portfolio demonstration | Synthetic data only         |

A separate staging environment may be introduced when the deployment complexity justifies it.

### Environment isolation

Each environment should have its own:

- Configuration.
- Database.
- Object-storage location.
- Authentication configuration.
- Service identities.
- Secrets.
- Logging context.

Do not let CI or local development accidentally write to the production database or production storage bucket.

## 4. Local Development

### Prerequisites

Install supported versions of:

- Node.js.
- pnpm.
- Python.
- The selected Python dependency manager.
- Docker and Docker Compose.
- Git.
- Terraform, if working on infrastructure.

Run `pnpm install` from the repository root to install JavaScript dependencies and set up Husky Git hooks. Before each commit, lint-staged runs Ruff on staged backend Python files and ESLint/Prettier on staged frontend source files; these local checks do not replace CI.

Pin or document supported versions. Do not assume that the newest release of every tool is compatible with the repository.

### Suggested local services

Run these services with Docker Compose (`docker compose up --build`):

- Next.js frontend (`localhost:3000`).
- FastAPI API (`localhost:8000`).
- PostgreSQL and Redis.
- Celery worker and outbox dispatcher.
- A one-shot migration service that completes before the API and worker start.

Compose persists PostgreSQL data and local uploads in named volumes. Use
`docker compose down` to stop the stack without deleting those volumes, or
`docker compose down --volumes` to remove the development data. The frontend's
browser API URL can be changed at build time with
`NEXT_PUBLIC_API_BASE_URL=http://localhost:8000/api/v1`. Server-side frontend
requests use `API_INTERNAL_BASE_URL`, which Compose sets to the backend service
address.

The frontend Docker build uses the repository-level pnpm workspace lockfile and
Next.js standalone output. For running application processes directly on the
host, use the package-specific commands below instead.

### Example `.env.example`

```dotenv
# Frontend
NEXT_PUBLIC_API_BASE_URL=http://localhost:8000/api/v1

# API
APP_ENV=development
DATABASE_URL=postgresql://eeg-lab:eeg-lab@localhost:5432/eeg-lab
REDIS_URL=redis://localhost:6379/0
LOG_LEVEL=DEBUG

# Storage
STORAGE_BACKEND=local
LOCAL_STORAGE_PATH=./.local-data/uploads

# Authentication
AUTH_MODE=development
AUTH_TOKEN_SECRET=insecure-development-secret-change-me
AUTH_JWT_ALGORITHMS=HS256
AUTH_TOKEN_ISSUER=eeg-lab
AUTH_TOKEN_AUDIENCE=eeg-lab-api
AUTH_COOKIE_NAME=eeg-lab_session
AUTH_COOKIE_SAMESITE=lax
DEV_ORGANISATION_NAME=Development Organisation
```

These values are examples for local development only. Production must use managed credentials and secure configuration. When `APP_ENV=production`, the API refuses to start with `AUTH_MODE=development`, a default or short `AUTH_TOKEN_SECRET`, or an asymmetric `AUTH_JWT_ALGORITHMS` value without `AUTH_PUBLIC_KEY`.

If the selected database driver uses a different connection-string format or separate connection settings, document and implement that format consistently.

### Local setup workflow

The exact commands depend on the package scripts and dependency manager selected for the repository. Document the final commands in the README and Makefile.

The setup process should perform these steps:

1. Install frontend dependencies.
2. Install backend and worker dependencies.
3. Start PostgreSQL and Redis.
4. Apply database migrations.
5. Generate synthetic EEG fixtures.
6. Start the API.
7. Start the worker.
8. Start the frontend.
9. Run the smoke tests.

### Local development acceptance criteria

- A new developer can reproduce the environment.
- No production credentials are required.
- Migrations work against a fresh database.
- The frontend can communicate with the API.
- The worker can process a submitted analysis.
- Synthetic uploads can be validated.
- The full analysis workflow can be demonstrated locally.

## 5. Containerisation

Use Docker images for the API and worker. Containerise the frontend if that matches the chosen Next.js deployment model.

### Container requirements

- Use a maintained, supported base image.
- Pin dependency versions.
- Prefer multi-stage builds where useful.
- Run as a non-root user.
- Avoid including development dependencies in production images unnecessarily.
- Do not copy `.env` files or credentials into images.
- Avoid placing scientific recordings in container layers.
- Use a predictable working directory.
- Define health checks where appropriate.
- Handle termination signals so workers can shut down cleanly.

### Example container layout

```text
containers/
├── web.Dockerfile
├── api.Dockerfile
└── worker.Dockerfile
```

The API and worker may share a Python dependency base while retaining separate runtime commands and permissions.

Do not assume that sharing an image means sharing the same execution role.

## 6. Google Cloud Architecture

The proposed initial production architecture is:

```mermaid id="p83k8x"
flowchart TD
    User["User's browser"]
    Web["Next.js frontend"]
    API["FastAPI on Cloud Run"]
    DB["Cloud SQL for PostgreSQL"]
    Storage["Private Cloud Storage"]
    Redis["Managed Redis-compatible broker"]
    Worker["Analysis worker"]
    Secrets["Secret Manager"]
    Logs["Cloud Logging and Monitoring"]
    CI["GitHub Actions"]
    TF["Terraform"]

    User --> Web
    Web --> API
    API --> DB
    API --> Storage
    API --> Redis
    Redis --> Worker
    Worker --> DB
    Worker --> Storage
    API --> Secrets
    Worker --> Secrets
    API --> Logs
    Worker --> Logs
    CI --> Web
    CI --> API
    CI --> Worker
    TF --> Web
    TF --> API
    TF --> DB
    TF --> Storage
    TF --> Redis
```

This is a logical architecture, not a guarantee that every named service is available in every region or suitable for every budget.

### Frontend

Deploy the Next.js application to Cloud Run or another compatible hosting service.

If deployed on Cloud Run, verify the application's server-rendering requirements, container startup behaviour, and port configuration.

Use HTTPS for public traffic.

### API

Deploy FastAPI as a Cloud Run service.

Requirements:

- Configure an appropriate request timeout.
- Set concurrency and instance limits deliberately.
- Configure health checks.
- Apply request-size limits.
- Use structured logs.
- Access PostgreSQL and object storage using the service identity.
- Keep secrets in Secret Manager or an equivalent managed secret facility.
- Avoid storing durable state on the container filesystem.

Do not perform long-running scientific computations inside an HTTP request handler.

### PostgreSQL

Use Cloud SQL for PostgreSQL if the selected budget and availability requirements justify it.

Requirements:

- Avoid public database exposure where practical.
- Restrict access to approved service identities and network paths.
- Configure backups.
- Document recovery objectives.
- Monitor connections and resource usage.
- Apply migrations through a controlled release process.
- Configure connection pooling appropriately for serverless concurrency.

### Object storage

Use a private Cloud Storage bucket for source recordings.

Requirements:

- Block public access.
- Use least-privilege IAM permissions.
- Use server-generated object keys.
- Apply upload size and expiry policies.
- Avoid permanent public URLs.
- Use short-lived signed URLs only where required.
- Record object identity and checksum metadata.
- Define retention and cleanup behaviour.

### Background worker

Deploy the Celery worker separately from the API.

The worker should:

- Process queued analyses.
- Load validated recordings.
- Enforce resource limits.
- Persist results and state transitions.
- Retry only recoverable failures.
- Avoid logging sensitive payloads.
- Shut down gracefully.
- Recover from duplicate or delayed task messages.

### Redis-compatible broker

Select a managed broker that is compatible with the required Celery transport and semantics.

Verify supported Redis protocol features, networking, authentication, TLS, persistence, and operational limits before committing to a provider.

Do not assume that every managed Redis offering supports every Celery configuration.

### Important deployment distinction

Cloud Run request-driven services may scale down when idle. Background workers require an execution model that continues to process queued work reliably.

Choose and test the worker deployment model explicitly. Do not assume that deploying a normal request-driven container automatically provides a continuously running Celery worker.

If the required managed-service combination is too expensive or operationally complex for the portfolio MVP, use a simpler documented deployment while preserving the application boundaries.

## 7. Terraform Infrastructure

Terraform should define the infrastructure required by the selected production environment.

### Suggested structure

```text
infrastructure/
└── terraform/
    ├── modules/
    │   ├── api/
    │   ├── database/
    │   ├── storage/
    │   ├── worker/
    │   └── networking/
    └── environments/
        ├── dev/
        └── prod/
```

Keep modules small and introduce them only where reuse or separation of concerns is valuable.

### Infrastructure responsibilities

Terraform should manage, where supported:

- Cloud Run services.
- Cloud SQL configuration.
- Storage buckets.
- IAM bindings.
- Networking.
- Secret resource definitions and access policies.
- Logging and monitoring configuration.
- Budget alerts where available.
- Required APIs and service identities.

Secret values should not be committed to Terraform files or checked-in variable files.

Avoid placing secret values in Terraform state. Marking a variable as sensitive does not prevent it from being stored in state.

### Terraform state

Use a remote state backend with access controls, versioning or recovery support where available, and a documented locking strategy.

Requirements:

- Keep state out of Git.
- Restrict who can read and modify state.
- Separate environments.
- Review plans before applying changes.
- Avoid concurrent conflicting applies.
- Document state recovery.

### Infrastructure acceptance criteria

- Terraform validates.
- Plans are reviewed.
- IAM follows least privilege.
- Public exposure is intentional.
- Storage is private by default.
- Database networking is restricted.
- State is protected.
- Teardown instructions are tested for disposable environments.

## 8. Configuration and Secrets

### Configuration

Keep non-secret environment-specific settings in explicit configuration.

Examples:

- Environment name.
- Log level.
- API URL.
- Resource limits.
- Supported file formats.
- Analysis concurrency.
- Feature flags.

### Secrets

Use Secret Manager or an equivalent managed secret facility for:

- Database credentials where credentials are required.
- Broker authentication.
- Authentication-provider secrets.
- Third-party API credentials.

Use workload identity or service identities wherever possible to avoid long-lived service-account keys.

### Rules

- Never commit secrets.
- Never log secret values.
- Never expose server secrets through `NEXT_PUBLIC_*` variables.
- Rotate credentials according to a documented policy.
- Use separate credentials for local, CI, and production.
- Grant each service access only to the secrets it requires.

## 9. Database Migrations

Use versioned `.up.sql` and `.down.sql` files under `packages/backend/migrations/versions/` for schema changes. Apply pending SQL migrations with `uv run python -m app.db.migrations upgrade`.

### Deployment order

A typical release should:

1. Build and test the application images.
2. Review the migration plan.
3. Apply a backward-compatible migration.
4. Deploy compatible application code.
5. Run smoke tests.
6. Verify health and error metrics.
7. Retain a documented recovery option.

For changes requiring multiple releases, use expand-and-contract migrations:

1. Add the new schema while retaining compatibility.
2. Deploy code that supports the transition.
3. Migrate data if required.
4. Switch reads and writes.
5. Remove obsolete schema in a later release.

Do not assume rolling back an application image will automatically reverse a database migration safely.

Avoid destructive automatic schema changes during application startup.

## 10. CI/CD with GitHub Actions

Use GitHub Actions for validation and deployment.

### Pull request pipeline

Run:

- Frontend linting.
- Frontend type checking.
- Frontend tests.
- Backend linting and tests.
- Scientific correctness tests.
- PostgreSQL integration tests.
- Worker tests.
- Production builds.
- Migration checks.
- Terraform formatting and validation.
- Security checks where configured.

### Deployment pipeline

A production workflow should:

1. Trigger only from approved branches or releases.
2. Run required checks.
3. Build immutable images.
4. Publish images to a container registry.
5. Authenticate using workload identity federation where supported.
6. Apply approved infrastructure changes when needed.
7. Run database migrations safely.
8. Deploy the API, worker, and frontend.
9. Execute smoke tests.
10. Report deployment status.

Prefer immutable image tags based on commit SHA or image digest. Do not rely only on a mutable `latest` tag.

### Deployment permissions

- Use protected production environments.
- Restrict who can approve releases.
- Keep deployment permissions minimal.
- Separate pull-request validation from production credentials.
- Do not grant untrusted forked pull requests production secrets.

## 11. Health Checks and Readiness

Implement separate health endpoints.

### Liveness

`GET /health/live`

Indicates that the API process is alive and able to respond.

This endpoint should not fail solely because an external dependency is temporarily unavailable.

### Readiness

`GET /health/ready`

Indicates whether the API can accept work under its defined readiness policy.

Check required dependencies appropriately, such as database connectivity. Avoid expensive or unbounded checks.

Do not expose credentials, internal network details, or detailed dependency errors through public health responses.

### Worker health

Track:

- Worker process availability.
- Last successful task.
- Queue depth where available.
- Job age.
- Failed task count.
- Retry count.
- Stale running jobs.

A healthy HTTP endpoint does not prove that the asynchronous processing pipeline is healthy.

## 12. Logging, Metrics, and Alerts

### Logging

Use structured logs with fields such as:

- Timestamp.
- Severity.
- Service.
- Environment.
- Request or correlation ID.
- Route template.
- Resource ID.
- Job ID.
- Duration.
- Safe error code.

Do not log raw recordings, signed URLs, access tokens, or unnecessary participant information.

### Metrics

Track at least:

- API request count and latency.
- API error rate.
- Database connection utilisation.
- Upload validation failures.
- Analysis queue depth.
- Job execution duration.
- Job success and failure counts.
- Retry counts.
- Worker availability.
- Storage consumption.

### Alerts

Start with a small number of actionable alerts:

- Sustained API failures.
- Repeated database connection failures.
- A growing queue of old jobs.
- Repeated worker crashes.
- Unexpectedly high resource consumption.
- Backup or migration failures where monitoring is available.

Avoid alerts that trigger on harmless short-lived fluctuations.

## 13. Backups and Recovery

Define a recovery policy for every durable data store.

### PostgreSQL

Document:

- Backup frequency.
- Retention.
- Recovery procedure.
- Who can restore.
- Recovery point objective.
- Recovery time objective.

Verify the actual backup capabilities of the selected service and configuration.

### Object storage

Document:

- Recording retention.
- Versioning or generation behaviour where enabled.
- Deletion policy.
- Recovery options.
- How database metadata maps to stored objects.

### Recovery testing

A backup is not proven useful until restoration has been tested.

For a portfolio project:

- Document the restore procedure.
- Perform a test restore in a disposable environment.
- Verify that representative metadata and synthetic recording references remain consistent.
- Record any limitations.

## 14. Cost Management

A portfolio project should not incur unnecessary ongoing cloud costs.

Before deployment:

- Estimate recurring database, storage, compute, networking, logging, and broker costs.
- Prefer the smallest suitable resource configuration.
- Set budget notifications where supported.
- Configure service scaling limits.
- Limit worker concurrency.
- Set upload size and retention limits.
- Avoid keeping unused environments running.
- Document which services continue to incur charges while idle.

A budget alert is not necessarily a spending cap.

Review current regional pricing and free-tier eligibility before provisioning resources. Do not assume that a service remains free simply because a demonstration is small.

## 15. Production Readiness Checklist

### Application

- [ ] Production build succeeds.
- [ ] API and worker images are immutable.
- [ ] Authentication is enabled.
- [ ] Tenant isolation tests pass.
- [ ] Upload limits are configured.
- [ ] Worker retries and timeouts are configured.
- [ ] Secrets are managed securely.
- [ ] Health checks work.

### Database and storage

- [ ] Migrations are reviewed.
- [ ] Database access is restricted.
- [ ] Backups are enabled as required.
- [ ] Restore instructions exist.
- [ ] Object storage is private.
- [ ] Retention and cleanup policies are defined.

### Operations

- [ ] Structured logs are available.
- [ ] Critical metrics are monitored.
- [ ] Alerts are actionable.
- [ ] Deployment smoke tests pass.
- [ ] Cost controls are configured.
- [ ] Rollback and recovery limitations are documented.

### Portfolio demonstration

- [ ] Only synthetic data is used.
- [ ] Demo credentials are not privileged production credentials.
- [ ] The README explains limitations.
- [ ] The primary workflow works in the deployed environment.
- [ ] A recruiter can understand the architecture and run the project locally.

## 16. Rollback Strategy

Application rollback and data rollback are separate operations.

### Application rollback

Keep previous immutable application images available. If a deployment fails, restore the previous compatible version and run smoke tests.

### Database rollback

Do not automatically reverse a migration without checking its effects.

Prefer backward-compatible migrations that allow the application to be rolled back without losing data.

For destructive changes, define a specific recovery plan before deployment.

### Worker compatibility

During a rollout, old and new workers may temporarily coexist.

Ensure that:

- Job payloads remain compatible during the transition.
- Algorithm versions are explicit.
- Result schemas are versioned.
- Workers cannot overwrite results from incompatible executions.
- Queued jobs remain processable after deployment.

## 17. Teardown and Resource Cleanup

Provide a documented teardown process for disposable environments.

Before deleting resources:

1. Identify the environment.
2. Confirm that it contains no data that must be retained.
3. Check for storage objects, database backups, and Terraform state.
4. Review the destruction plan.
5. Confirm that shared resources will not be removed.
6. Apply the approved teardown.
7. Verify that the intended resources were removed.
8. Check for remaining billable resources.

Do not automatically destroy production resources as part of an ordinary deployment failure handler.

Object storage, backups, container images, logging, networking, and managed databases may have independent retention or billing behaviour. Verify each service explicitly.

## 18. Troubleshooting Guide

### API cannot connect to PostgreSQL

Check:

- Database availability.
- Connection-string format.
- Network configuration.
- Authentication and permissions.
- Connection-pool limits.
- Secret configuration.
- Database connection exhaustion.

Do not print passwords or full connection strings into logs.

### Worker does not process jobs

Check:

- Broker availability.
- Broker authentication and TLS.
- Worker deployment status.
- Queue configuration.
- Task registration.
- Dispatch failures in the outbox.
- Queue depth.
- Worker concurrency and resource limits.

Inspect job and outbox records before manually resubmitting work.

### Dataset upload remains pending

Check:

- Upload-session state.
- Signed URL expiry.
- Object existence.
- Object size and checksum.
- Validation-worker health.
- Validation error records.
- Object-storage permissions.

Do not manually mark the dataset ready without verifying the recording.

### Analysis remains queued

Check:

- Outbox dispatch status.
- Broker connectivity.
- Worker availability.
- Retry policy.
- Resource limits.
- Job state and lease information.

Use the documented recovery procedure rather than directly modifying production database rows without an approved process.

### Deployment succeeds but the application fails

Check:

- Environment variables.
- Health-check configuration.
- API base URL.
- Database migration status.
- Service identity permissions.
- Frontend/backend contract compatibility.
- Worker and broker connectivity.

## 19. Definition of Done

Deployment is complete when:

1. The application can be run locally using documented instructions.
2. CI validates the codebase.
3. Infrastructure is defined reproducibly.
4. Production secrets are not stored in source control.
5. The API and worker have appropriate service permissions.
6. Database and object-storage access are restricted.
7. Migrations have a safe deployment strategy.
8. Logs and basic operational metrics are available.
9. Backups and recovery procedures are documented.
10. Cloud costs and cleanup procedures are documented.
11. A deployed synthetic-data workflow passes smoke tests.
12. Known operational and scientific limitations are clearly documented.
