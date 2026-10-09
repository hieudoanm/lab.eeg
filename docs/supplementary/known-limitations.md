# Known Limitations

This document records current limitations of the EEG Lab system. Each limitation should have a tracked issue or decision record for future resolution.

## Scientific Limitations

| Limitation                                      | Impact                                                            | Tracking  |
| ----------------------------------------------- | ----------------------------------------------------------------- | --------- |
| Only descriptive statistics implemented         | No spectral analysis, connectivity, or advanced signal processing | Phase 7+  |
| Single analysis type (`descriptive_statistics`) | Cannot compare algorithms or versions                             | Phase 7+  |
| Fixed algorithm version `1.0.0`                 | No versioning of analysis methods                                 | Phase 7+  |
| No clinical validation                          | Results are for research exploration only                         | By design |
| No reference datasets or normative data         | Cannot contextualize results                                      | Future    |
| Synthetic data only for testing                 | No real EEG data in repository                                    | By design |

## Technical Limitations

### API & Backend

| Limitation                                                      | Impact                                                             | Tracking                     |
| --------------------------------------------------------------- | ------------------------------------------------------------------ | ---------------------------- |
| Development auth mode only (`AUTH_MODE=development`)            | No OIDC/SSO integration; tokens are HMAC-signed with shared secret | Phase 8+                     |
| No rate limiting on API endpoints                               | Potential for abuse / DoS                                          | Phase 8+                     |
| Request body limit hardcoded (1 MB default, 256 MB for uploads) | Large metadata payloads rejected                                   | Configurable but not dynamic |
| No API versioning strategy                                      | Breaking changes require coordination                              | Phase 9+                     |
| OpenAPI docs disabled in production                             | No self-service API exploration in prod                            | By design (security)         |
| No request/response compression                                 | Larger payloads for statistics results                             | Phase 8+                     |

### Database

| Limitation                                 | Impact                                             | Tracking           |
| ------------------------------------------ | -------------------------------------------------- | ------------------ |
| No soft delete / archival for all entities | `archived` status only on studies/organisations    | Phase 8+           |
| No partitioning on large tables            | `audit_events`, `outbox_events` may grow unbounded | Phase 8+           |
| No read replicas                           | All reads go to primary                            | Phase 9+           |
| UUID primary keys (random)                 | Index fragmentation, larger indexes                | Consider ULID/TSID |

### Background Processing

| Limitation                                       | Impact                                        | Tracking                              |
| ------------------------------------------------ | --------------------------------------------- | ------------------------------------- |
| Dispatcher must be running for job delivery      | If dispatcher stops, jobs pile up in `queued` | Phase 8: add cron fallback            |
| No dead-letter queue for exhausted outbox events | Exhausted events parked with 100-year delay   | Manual intervention needed            |
| Worker concurrency not configurable              | Fixed to Celery worker pool size              | Phase 8+                              |
| No priority queue                                | All jobs FIFO                                 | Phase 8+                              |
| Cancellation only for `queued` jobs              | Running jobs cannot be interrupted            | Requires `cancel_requested_at` column |

### Storage

| Limitation                          | Impact                                             | Tracking             |
| ----------------------------------- | -------------------------------------------------- | -------------------- |
| Local filesystem adapter only       | No horizontal scaling, single point of failure     | Phase 9: GCS/S3      |
| No direct-to-object-storage uploads | All uploads proxied through API (backend-mediated) | Phase 8: signed URLs |
| No multipart upload support         | Large files may timeout                            | Phase 8+             |
| No CDN for recording access         | All downloads through API                          | Phase 9+             |

### Frontend

| Limitation                              | Impact                              | Tracking                             |
| --------------------------------------- | ----------------------------------- | ------------------------------------ |
| No offline support                      | Requires constant connectivity      | Phase 9+                             |
| No real-time updates (WebSockets/SSE)   | Polling for job status              | Phase 8+                             |
| Limited keyboard navigation in charts   | Accessibility gap                   | Phase 8+                             |
| No data export (CSV/JSON)               | Cannot download analysis results    | Phase 7+                             |
| Chart downsampling fixed at 5000 points | May lose detail for long recordings | Configurable but not user-controlled |

### Security

| Limitation                             | Impact                             | Tracking |
| -------------------------------------- | ---------------------------------- | -------- |
| No CSP headers                         | XSS risk if JSON injection occurs  | Phase 8+ |
| No HSTS in development                 | Cookie security only in production | Phase 8+ |
| No secrets rotation automation         | Manual `AUTH_TOKEN_SECRET` updates | Phase 9+ |
| No audit log export / SIEM integration | Compliance gap                     | Phase 8+ |
| No penetration testing in CI           | Security regressions undetected    | Phase 8+ |

### Observability

| Limitation                      | Impact                                   | Tracking |
| ------------------------------- | ---------------------------------------- | -------- |
| No distributed tracing          | Cannot trace request across API → worker | Phase 8+ |
| No log aggregation / ELK / Loki | Logs only on stdout                      | Phase 8+ |
| No alerting rules defined       | No automated incident detection          | Phase 8+ |
| No SLO/SLI definitions          | No reliability targets                   | Phase 8+ |

### Testing

| Limitation                                   | Impact                 | Tracking |
| -------------------------------------------- | ---------------------- | -------- |
| No contract testing between frontend/backend | Type drift possible    | Phase 8+ |
| No load/performance tests                    | Capacity unknown       | Phase 8+ |
| No chaos engineering                         | Failure modes untested | Phase 9+ |
| E2E tests require live API                   | Not fully hermetic     | Phase 8+ |

## Operational Limitations

| Limitation                              | Impact                   | Tracking |
| --------------------------------------- | ------------------------ | -------- |
| No database migration rollback tested   | Down migrations untested | Phase 8+ |
| No blue/green deployment                | Downtime on deploy       | Phase 9+ |
| No infrastructure as code for local dev | Manual Docker Compose    | Phase 9+ |
| No runbook for common incidents         | Manual troubleshooting   | Phase 8+ |
| No capacity planning guidance           | Scaling reactive         | Phase 9+ |

## Data Model Limitations

| Limitation                       | Impact                           | Tracking |
| -------------------------------- | -------------------------------- | -------- |
| No dataset versioning            | Cannot track recording changes   | Phase 7+ |
| No analysis comparison / diff    | Cannot compare re-runs           | Phase 7+ |
| No annotation / label support    | Cannot mark artefacts            | Phase 7+ |
| Channel metadata fixed at upload | Cannot add/rename channels later | Phase 7+ |
| No subject / session hierarchy   | Flat dataset structure           | Phase 7+ |

## Documentation Gaps

| Gap                                                                    | Impact                      |
| ---------------------------------------------------------------------- | --------------------------- |
| API reference synced manually (`docs/api/api-spec.json`), no generator | Spec can drift from the app |
| No workflow screenshots in the README                                  | Portfolio presentation gap  |
| No hosted demo environment                                             | Reviewers must run locally  |

## Resolved Limitations (Historical)

| Limitation                   | Resolved In | Resolution                       |
| ---------------------------- | ----------- | -------------------------------- |
| No async analysis pipeline   | Phase 6     | Celery + outbox pattern          |
| No idempotency on submission | Phase 6     | `Idempotency-Key` header         |
| No result provenance         | Phase 6     | `analysis_results.provenance`    |
| No stale job recovery        | Phase 6     | Lease-based reaper               |
| No synthetic data generation | Phase 5     | `app.scientific.synthetic`       |
| No upload validation         | Phase 5     | Synchronous finalisation         |
| No tenant isolation tests    | Phase 3     | `tests/security/test_tenancy.py` |

---

_This document is updated as limitations are resolved or new ones are discovered. See `docs/decisions/` for architectural context._
