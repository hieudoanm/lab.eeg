# EEG Lab — Agent and Contributor Guide

This file contains the repository conventions that automated agents and human contributors must follow.

## 1. Project

EEG Lab is a full-stack scientific SaaS demonstration for neuroscience research: studies, EEG recordings, interactive signal exploration, asynchronous descriptive analyses, and organisation-scoped access control.

Read `docs/` before implementing. The documents are normative:

| Document                         | Governs                                                |
| -------------------------------- | ------------------------------------------------------ |
| `docs/01-overview.md`            | Product vision and scope                               |
| `docs/02-requirements.md`        | Functional/non-functional requirements (`FR-`, `NFR-`) |
| `docs/03-user-stories.md`        | User goals and acceptance scenarios                    |
| `docs/04-features.md`            | User-visible behaviour                                 |
| `docs/05-architecture.md`        | Module boundaries, request lifecycle, tenancy          |
| `docs/06-data-model.md`          | Schema, constraints, state transitions                 |
| `docs/07-api-spec.md`            | REST contract, status codes, error envelope            |
| `docs/08-tech-stack.md`          | Allowed technologies and dependency policy             |
| `docs/09-design.md`              | UI tokens, layout, states, accessibility               |
| `docs/10-testing-strategy.md`    | Test layers, fixtures, acceptance criteria             |
| `docs/11-implementation-plan.md` | Phased delivery plan                                   |
| `docs/12-deployment.md`          | Operations and deployment                              |

When an implementation and a document disagree, stop and resolve the disagreement explicitly rather than silently choosing one.

## 2. Repository layout

```text
eeg-lab/
├── packages/
│   ├── frontend/         Next.js + React + TypeScript frontend
│   └── backend/          FastAPI application, SQL migrations, worker tasks
├── infrastructure/       Terraform
├── scripts/              Development and data-generation scripts
├── tests/                Cross-cutting end-to-end tests
├── docs/                 Normative project documentation
└── .github/workflows/    CI
```

Do not create empty placeholder modules. Prefer a small, coherent implementation.

## 3. Toolchain

| Area           | Tool                                                                          |
| -------------- | ----------------------------------------------------------------------------- |
| Python         | `uv` (lockfile committed), Python pinned in `packages/backend/pyproject.toml` |
| Node           | `pnpm`, version pinned in `.nvmrc`                                            |
| Backend lint   | `ruff check`, `ruff format`                                                   |
| Backend types  | `mypy`                                                                        |
| Backend tests  | `pytest`                                                                      |
| Frontend lint  | `eslint`, `prettier`                                                          |
| Frontend types | `tsc --noEmit`                                                                |
| Frontend tests | `vitest`, `@testing-library/react`, `playwright`                              |

Run the relevant checks through the root `Makefile`. Never weaken a check to make CI pass.

## 4. Backend conventions

### Structure

- `app/api/` — routes, dependencies, error handlers. Keep handlers thin.
- `app/core/` — configuration, logging, exceptions, security helpers.
- `app/db/` — connection pool and transaction helpers.
- `app/repositories/` — explicit SQL and row mapping only.
- `app/services/` — business operations coordinating repositories and adapters.
- `app/schemas/` — Pydantic request/response models.
- `app/scientific/` — deterministic NumPy computations.
- `app/storage/` — object-storage adapter.
- `app/workers/` — Celery application and tasks.

Never import route handlers into business logic. Never expose database rows directly as API responses.

### Database rules

1. All values are passed as query parameters. Never interpolate user input into SQL.
2. Dynamic identifiers come only from trusted allowlists.
3. Transaction boundaries are explicit and kept short. Do not hold a transaction open across object-storage or heavy CPU work.
4. Every tenant-owned query filters by a verified organisation context supplied by the application layer, never by a raw client value.
5. Every schema change ships as a reviewed, versioned SQL migration.
6. Connection pool usage is bounded; do not share one connection across unrelated tasks.

### API rules

1. Base path is `/api/v1`.
2. Responses use the error envelope from `docs/07-api-spec.md`.
3. UUIDs identify resources; timestamps are ISO 8601 UTC (`...Z`).
4. Unknown request fields are rejected on writes.
5. List endpoints use cursor pagination, deterministic ordering, and `MAX_PAGE_SIZE`.
6. Never return stack traces, SQL, credentials, object keys, or signed URLs.

### Security rules

1. Deny by default. Authenticate before authorising.
2. `404` is preferred over disclosing another tenant's resource.
3. Validate and bound every upload before parsing.
4. Never enable pickle loading for untrusted NumPy data.
5. Keep secrets out of the repository, logs, and frontend bundles.
6. Every authorisation path needs a negative test.

## 5. Scientific conventions

1. Array orientation is `(channels, samples)` and is enforced everywhere.
2. Descriptive statistics exclude non-finite samples and report `valid_sample_count`.
3. Standard deviation uses the population convention (`ddof=0`).
4. A channel with no valid samples is an error, not a zero-filled result.
5. Display downsampling is never an input to analysis.
6. Every persisted result records dataset checksum, algorithm version, parameters, and result schema version.
7. Never describe descriptive statistics as clinical findings.

## 6. Testing conventions

1. Tests assert observable behaviour, not internal implementation.
2. Scientific tests compare against independently computed expected values with explicit tolerances.
3. PostgreSQL-dependent tests use a real database, not SQLite or mocks.
4. Tenant isolation and role enforcement are always tested negatively.
5. Worker tests assume duplicate delivery and interruption.
6. Fixtures are synthetic and deterministic (fixed seeds). Never commit real recordings.
7. Mark tests that need external services so they can be skipped deliberately, never silently.

## 7. Frontend conventions

1. Strict TypeScript, no `any`.
2. Design tokens from `docs/09-design.md`; avoid scattered arbitrary values.
3. Every data screen implements loading, empty, no-results, validation-error, permission-error, network-error, and unexpected-error states.
4. Never show success before the server confirms it.
5. Charts require labels, units where known, and an accessible textual alternative.
6. Hide unavailable actions for usability, but treat the server as the only authority.

## 8. Delivery rules

1. Keep every commit runnable.
2. Add tests alongside the behaviour they protect.
3. Update documentation when behaviour or scope changes.
4. Record significant decisions in `docs/decisions/`.
5. Do not commit generated data, credentials, or lock files belonging to Terraform state.
6. Do not claim a feature that is not implemented and tested.
