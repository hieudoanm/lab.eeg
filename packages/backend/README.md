# EEG Lab API

FastAPI application, PostgreSQL SQL migrations, and scientific processing for EEG Lab.

## Commands

Run these from the repository root:

```bash
make db-migrate   # apply pending SQL migrations
make api-run      # run the API
make test-api     # run pytest
make lint         # ruff + mypy
```

## Layout

```text
packages/backend/
├── app/
│   ├── api/          routes, dependencies, error handling
│   ├── core/         configuration, logging, exceptions
│   ├── db/           connection pool and transaction helpers
│   ├── repositories/ explicit SQL and row mapping
│   ├── schemas/      Pydantic request and response models
│   ├── services/     business operations
│   ├── scientific/   deterministic NumPy computations
│   ├── storage/      object-storage adapter
│   └── workers/      background task entry points
├── migrations/       Versioned SQL upgrade and downgrade files
└── tests/            unit, API, integration, and security tests
```

## Conventions

See `AGENTS.md` at the repository root. The governing documents are `docs/07-api-spec.md`, `docs/06-data-model.md`, and `docs/05-architecture.md`.
