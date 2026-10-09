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

