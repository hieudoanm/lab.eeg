# EEG Lab — Portfolio Case Study

**Project Type**: Full-stack scientific SaaS demonstration  
**Duration**: Phased implementation (Phases 0–10)  
**Role**: Solo engineer / portfolio project  
**Tech Stack**: Python 3.12, FastAPI, PostgreSQL, Redis, Celery, Next.js 16, React 19, TypeScript, Plotly.js, Tailwind CSS, daisyUI, Docker, Terraform, GitHub Actions

---

## Problem Statement

Neuroscience researchers need a secure, reproducible platform to:
1. Organise studies and EEG datasets with rich metadata
2. Upload and validate recordings with integrity guarantees
3. Run asynchronous descriptive analyses on large numerical arrays
4. Visualise results with scientific rigour (units, provenance, accessibility)
5. Collaborate within organisation-scoped workspaces

Existing tools are either:
- **Desktop-only** (no collaboration, no provenance)
- **Clinical/closed-source** (expensive, not extensible)
- **Generic cloud notebooks** (no domain-specific validation, no tenant isolation)

---

## Solution Overview

EEG Lab is a **multi-tenant neuroscience research platform** demonstrating production-grade engineering practices:

| Capability | Implementation |
|------------|----------------|
| **Secure multi-tenancy** | JWT auth, org-scoped queries, role-based access (owner/researcher/viewer) |
| **EEG ingestion** | Upload sessions → backend-mediated → checksum + manifest validation → lifecycle states |
| **Async analysis** | Transactional outbox + Celery/Redis → deterministic NumPy stats → provenance |
| **Scientific visualisation** | Plotly.js time-series with downsampling, channel selector, data-table fallback |
| **Reliability** | Structured logging, Prometheus metrics, bounded retries, stale-job recovery, E2E tests |
| **Quality gates** | Ruff, mypy, pytest (191), vitest (58), Playwright, Trivy, dependency audits |
| **Infrastructure** | Docker Compose (local), Terraform + GCP Cloud Run (target) |

---

## Key Engineering Decisions

### 1. Transactional Outbox over Native Celery Retries

**Problem**: Celery retries are in-memory; worker restart loses retry state. No visibility into retry count from API.

**Decision**: Persist job state in PostgreSQL (`attempt_count`, `max_attempts`, `lease_expires_at`). Worker claims job atomically, computes, commits result + state in one transaction. Requeues via new outbox event on transient failure.

**Trade-off**: Slightly more complex than `autoretry_for`, but:
- Retry logic visible in DB
- Survives worker/API restarts
- Bounded by `max_attempts` (default 3)
- Stale-job recovery via lease expiration

### 2. Idempotency Keys for Safe Submission

**Problem**: Network retries create duplicate jobs. Researchers may double-click "Submit".

**Decision**: Client generates `Idempotency-Key` header. Server reserves key per `(org, user, operation, key)` with request hash. Identical replay returns original job; different body → 409.

**Trade-off**: Client responsibility for key generation; documented in API spec.

### 3. NumPy `.npy` Only (No Pickle)

**Problem**: `np.load(allow_pickle=True)` is a remote code execution vector.

**Decision**: Enforce `allow_pickle=False` everywhere. Validate shape `(channels, samples)`, dtype, manifest at upload finalisation AND worker execution. Double verification prevents TOCTOU.

### 4. Provenance-First Results

**Problem**: Scientific results without context are irreproducible.

**Decision**: Every `analysis_result` stores:
- `dataset_checksum_sha256` (input identity)
- `algorithm_version` + `result_schema_version` (code identity)
- `parameters` (configuration)
- `sampling_frequency_hz`, `channel_count`, `sample_count` (acquisition context)

### 5. Structured JSON Logging with Correlation IDs

**Problem**: Distributed debugging across API → dispatcher → worker is hard.

**Decision**: `X-Request-Id` header propagated via contextvars. Worker binds `job_id`. All logs include both. Prometheus metrics tagged by route/task.

### 6. Real PostgreSQL in Tests (No SQLite)

**Problem**: SQLite doesn't enforce Postgres constraints, types, or concurrency behaviour.

**Decision**: All integration tests run against real PostgreSQL (Docker in CI, local dev). Fixtures use `TRUNCATE ... CASCADE` for isolation.

---

## Technical Highlights

### Backend (FastAPI + PostgreSQL)
- **Explicit SQL** — No ORM; hand-written queries in repositories
- **Tenant isolation** — Every query filters by verified `organisation_id`
- **SQL migrations** — Versioned PostgreSQL scripts for schema changes
- **Health checks** — `/live` (process), `/ready` (DB + Redis + storage)
- **Error envelope** — RFC 7807-inspired `{error: {code, message, details}}`

### Workers (Celery + Redis)
- **No result backend** — PostgreSQL is authoritative
- **Lease-based stale recovery** — `lease = max(job_lease, task_timeout + 30s)`
- **Dispatcher row locking** — `FOR UPDATE SKIP LOCKED` held across Redis publish
- **Bounded dispatch retries** — Exponential backoff → park at 100 years with error code

### Frontend (Next.js 16 + React 19)
- **Server Components** — Data fetching in RSC, client components for interactivity
- **Plotly.js** — `scattergl` for 5k+ points, downsampling, pan/zoom, hover
- **Accessible alternatives** — Data table toggle for every chart
- **Design tokens** — Centralised colours, spacing, status tones (never colour-only)

### Testing Strategy
| Layer | Tool | Count | Focus |
|-------|------|-------|-------|
| Unit (scientific) | pytest | 11 | Statistics vs independent calc |
| Unit (backend) | pytest | 30+ | Config, recording, synthetic, cleanup |
| API | pytest + HTTPX | 50+ | Auth, tenancy, CRUD, idempotency |
| Integration | pytest | 20+ | Upload, cleanup, analysis execution, recovery |
| Security | pytest | 10 | Cross-tenant, role enforcement |
| Unit (frontend) | vitest | 58 | Components, utils, API client |
| E2E | Playwright | 20+ | Full workflow, tenant isolation |

### CI/CD (GitHub Actions)
- **Backend**: Ruff, mypy, pytest, pip-audit
- **Frontend**: ESLint, Prettier, tsc, vitest, build, npm audit
- **Security**: Trivy filesystem scan → SARIF → GitHub Security tab
- **Concurrency**: Cancel in-progress on new push

---

## Challenges Overcome

| Challenge | Solution |
|-----------|----------|
| **Duplicate job delivery** | Idempotency keys + unique constraint on `analysis_results.job_id` + worker duplicate detection |
| **Worker crash mid-computation** | Lease expiration → recovery reaper requeues or fails with `analysis_stale` |
| **Recording corruption after upload** | SHA-256 verified at finalise AND at worker execution; size check |
| **Large array visualisation** | Downsampling to 5k points + data-table fallback; `scattergl` WebGL rendering |
| **Tenant isolation bugs** | Centralised `tenancy.require_role` + negative tests for every endpoint |
| **Schema drift** | Pydantic `extra="forbid"` on all request models; versioned SQL migrations for DB changes |

---

## Metrics & Outcomes

| Metric | Value |
|--------|-------|
| **Backend test coverage** | 191 tests (unit + API + integration) |
| **Frontend test coverage** | 58 tests (unit) + 20+ E2E scenarios |
| **Type safety** | mypy strict (105 files), tsc --noEmit |
| **Linting** | Ruff (0 errors), ESLint (0 errors) |
| **API endpoints** | 25+ (studies, datasets, uploads, analyses, auth, health) |
| **Database tables** | 11 (users, orgs, memberships, studies, datasets, upload_sessions, analysis_jobs, analysis_results, outbox_events, idempotency_keys, audit_events) |
| **Worker tasks** | `eeg-lab.run_analysis`, `eeg-lab.cleanup_uploads` |
| **Documentation** | 12 core docs + 3 decision records + diagrams + threat model |

---

## Limitations & Future Work

| Area | Limitation | Next Step |
|------|------------|-----------|
| **Auth** | Dev-mode only (HS256) | OIDC + asymmetric JWT (Phase 10) |
| **Analysis** | Only descriptive statistics | Spectral, connectivity, ML (Phase 7+) |
| **Storage** | Local FS only | GCS/S3 adapter + signed URLs (Phase 10) |
| **Real-time** | Polling for job status | WebSocket/SSE (Phase 8+) |
| **Rate limiting** | Body size only | Per-user/endpoint limits (Phase 10) |
| **Observability** | Metrics only | Distributed tracing, log aggregation (Phase 10) |
| **Deployment** | Local Docker only | Terraform + GCP Cloud Run (Phase 10) |

---

## Repository Structure

```
eeg-lab/
├── packages/
│   ├── backend/          # FastAPI, Celery, PostgreSQL, scientific computing
│   └── frontend/         # Next.js, React, Plotly.js, Tailwind
├── infrastructure/       # Terraform (GCP target)
├── docs/                 # 12 normative documents + decisions + diagrams
├── scripts/              # Dev utilities (dispatcher, cleanup, generate)
├── .github/workflows/    # CI pipeline
├── Makefile              # Unified task runner
└── AGENTS.md             # Contributor conventions
```

---

## Links

- **Source**: https://github.com/hieudoanm/eeg-lab
- **Documentation**: `docs/` (architecture, API spec, data model, testing strategy, decisions)
- **Architecture Diagrams**: `docs/architecture-diagrams.md` (Mermaid)
- **Threat Model**: `docs/threat-model.md` (STRIDE)
- **Known Limitations**: `docs/known-limitations.md`

---

*This case study emphasises engineering process: how requirements drove architecture, how failure modes shaped the outbox pattern, how scientific rigour informed the provenance model, and how testing strategy catches cross-tenant bugs. The code is the ultimate documentation — start with `packages/backend/app/` and `packages/frontend/src/`.*