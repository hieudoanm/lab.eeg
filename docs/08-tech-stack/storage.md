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

