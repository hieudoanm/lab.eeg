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

