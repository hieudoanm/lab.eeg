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

