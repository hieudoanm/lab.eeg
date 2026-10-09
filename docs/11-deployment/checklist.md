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

