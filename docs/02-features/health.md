## 12. Operational Health

**Priority:** P0

### Description

Developers and operators need to determine whether the API and its dependencies are functioning.

### Health endpoints

- `/healthz`: confirms that the API process is alive.
- `/readyz`: reports whether essential dependencies are ready.

Health endpoints must not reveal secrets or connection strings.

### Logs

Use structured logs with:

- Timestamp.
- Severity.
- Request or job identifier.
- Relevant organisation identifier when safe.
- Operation name.
- Outcome.
- Error classification.

Avoid logging raw EEG arrays, credentials, signed URLs, or sensitive payloads.

