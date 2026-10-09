## 15. External Service Boundaries

Wrap external integrations behind small interfaces where this improves testability.

Examples:

- `ObjectStorage`: upload, read, inspect, and delete objects.
- `AnalysisDispatcher`: publish or schedule a durable job.
- `AuthenticationProvider`: validate identity or resolve session.
- `Clock`: provide deterministic time in tests where useful.

Do not introduce a generic plugin framework or extensive dependency-injection system without a concrete need.

The goal is to make infrastructure replaceable and testable without hiding the important application behaviour.

## 16. Observability

### Structured logs

Include fields such as:

- Timestamp.
- Severity.
- Service name.
- Request or job ID.
- Operation.
- Outcome.
- Error classification.
- Duration.
- Organisation identifier where safe.

### Metrics

Track useful operational measurements:

- Request latency.
- API error rate.
- Database connection utilisation.
- Upload failures.
- Queue depth.
- Job execution duration.
- Job retry count.
- Job failure rate.
- Stale job count.

### Correlation

A job should retain enough context to correlate its API submission, dispatch, worker execution, and result persistence.

Do not put secrets, access tokens, signed URLs, or raw EEG payloads in logs.

## 17. Deployment Architecture

### Local development

Use Docker Compose for PostgreSQL, Redis, and any supporting local services.

Run the frontend, API, and worker using documented commands or containers.

The development setup should not require access to production credentials.

### Production

A practical initial deployment may use:

- Next.js hosted through a suitable managed web platform.
- FastAPI deployed as a managed container.
- A separate worker deployment.
- Managed PostgreSQL.
- Private object storage.
- A compatible managed Redis broker.
- Managed secrets and logging.

Terraform should define the required infrastructure and access policies.

The chosen services must support the connection, networking, worker, and durability requirements of the application.

## 18. Failure Scenarios

| Failure                           | Expected behaviour                                                              |
| --------------------------------- | ------------------------------------------------------------------------------- |
| PostgreSQL unavailable            | Requests requiring database access fail safely; readiness reports unavailable   |
| Redis unavailable                 | Job dispatch is delayed or retried; accepted jobs remain visible in PostgreSQL  |
| Worker crashes                    | Job recovery identifies stale execution and safely retries eligible work        |
| Object storage unavailable        | Upload or retrieval fails clearly; metadata remains consistent                  |
| Upload interrupted                | Upload remains incomplete and can be retried or cleaned up                      |
| Result write fails                | Job is not marked successful                                                    |
| Duplicate task delivered          | Worker state checks prevent conflicting duplicate results                       |
| Session expires                   | User is asked to re-authenticate or renew the session                           |
| Organisation membership revoked   | Protected operations are denied after revocation takes effect                   |
| Frontend request becomes obsolete | Cancellation or request identity prevents stale results replacing current state |

## 19. Security Boundaries

Treat these as separate trust zones:

1. Browser and client-controlled state.
2. Authenticated API.
3. Relational database.
4. Object storage.
5. Message broker.
6. Background worker.
7. Cloud control plane.

Security requirements:

- Validate all external input.
- Authenticate before protected access.
- Authorise each resource operation.
- Use least-privilege service identities.
- Restrict network access to internal services.
- Use TLS for external service communication.
- Protect secrets and signing credentials.
- Bound CPU, memory, file sizes, and execution time.
- Keep uploaded files non-executable.
- Never execute uploaded code.
- Avoid leaking data through logs, error responses, caches, or object URLs.

## 20. Architecture Decision Records

Document significant decisions under `docs/decisions/`.

At minimum:

- Authentication provider and session model.
- EEG file format and partial-read strategy.
- Job dispatch and transactional outbox.
- Explicit SQL and repository boundaries.
- Object-storage upload lifecycle.
- Worker hosting and Redis compatibility.
- Organisation isolation strategy.
- Deployment topology.

Each record should state the context, decision, alternatives, consequences, and criteria for reconsideration.

## 21. Architecture Definition of Done

The architecture is implemented when:

1. The API and worker run independently.
2. The application has explicit module boundaries.
3. Tenant isolation is enforced and tested.
4. PostgreSQL is authoritative for jobs and results.
5. Background task dispatch has a documented reliability strategy.
6. Duplicate task delivery cannot corrupt results.
7. The repository's Husky pre-commit hook runs lint-staged on staged backend and frontend source; CI remains the full validation authority.
8. Large recordings are handled within configured resource limits.
9. Source data and result provenance are preserved.
10. Cross-service failure recovery is documented and tested.
11. Structured logging supports investigation.
12. Secrets and private recordings are protected.
13. Local development and deployment follow documented architecture.
