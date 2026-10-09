## 15. Repository Organisation

A suggested repository structure:

```text
eeg-lab/
├── packages/
│   ├── backend/
│   │   ├── app/
│   │   │   ├── api/
│   │   │   ├── core/
│   │   │   ├── db/
│   │   │   ├── models/
│   │   │   ├── repositories/
│   │   │   ├── schemas/
│   │   │   ├── services/
│   │   │   ├── workers/
│   │   │   └── main.py
│   │   ├── migrations/
│   │   │   └── versions/
│   │   │       ├── 0001_initial_schema.up.sql
│   │   │       ├── 0001_initial_schema.down.sql
│   │   │       ├── 0002_add_upload_sessions_manifest.up.sql
│   │   │       └── 0002_add_upload_sessions_manifest.down.sql
│   │   ├── tests/
│   │   └── pyproject.toml
│   ├── frontend/
│   │   ├── src/
│   │   ├── public/
│   │   ├── tests/
│   │   └── package.json
│   └── api-client/          proposed shared behaviour contract
├── infra/
│   └── terraform/
├── scripts/
├── docs/
├── docker-compose.yml
├── Makefile
├── .github/
│   └── workflows/
└── README.md
```

This is a proposed structure, not a requirement to create every directory immediately.

Prefer a small, coherent implementation over empty placeholder modules.

## 16. Configuration and Secrets

Use environment variables or a documented configuration mechanism.

Configuration should include:

- Environment name.
- API base URL.
- Database connection settings.
- Redis broker URL.
- Object-storage configuration.
- Authentication provider settings.
- Upload and processing limits.
- Logging level.
- CORS origins.

Requirements:

- Validate required configuration at startup.
- Fail clearly when mandatory configuration is missing.
- Never commit real credentials.
- Provide an `.env.example` with placeholders only.
- Keep local development secrets out of version control.
- Use managed identities and a secrets manager in deployed environments where supported.

## 17. Deferred Technologies

Do not introduce these technologies in the initial MVP unless a demonstrated requirement justifies them:

- Kubernetes.
- A service mesh.
- Kafka.
- Multiple databases.
- A heavyweight ORM.
- A second backend language.
- Distributed tracing infrastructure beyond what the initial operational needs justify.
- GPU-based computation.
- A dedicated feature store.
- Large-scale machine-learning orchestration.
- Real-time EEG streaming infrastructure.

These may be relevant to a larger research platform, but they increase implementation and operational cost before the core product is proven.

## 18. Technology Decision Record Requirements

Record major architectural decisions in `docs/decisions/`.

Each decision should include:

- Context.
- Decision.
- Alternatives considered.
- Reasons for the choice.
- Trade-offs.
- Consequences.
- Conditions that would justify revisiting the decision.

Create decision records for at least:

1. Explicit SQL versus ORM.
2. Recording file format.
3. Background job architecture.
4. Authentication approach.
5. Object-storage strategy.
6. Deployment and queue topology.

## 19. Definition of Done

The technology stack is correctly implemented when:

1. Frontend and backend have independent build and test commands.
2. Type checking and linting are enforced.
3. PostgreSQL schema changes use migrations.
4. Database queries are parameterised.
5. Background processing is separated from API request handling.
6. Large scientific files are stored outside PostgreSQL.
7. Configuration and secrets are handled safely.
8. Local development is reproducible.
9. CI validates the critical application workflows.
10. Cloud deployment can be reproduced through documented infrastructure configuration.
11. Every selected technology has a clear responsibility.
12. The implementation does not introduce deferred technologies without a documented reason.

### 4.5 Frontend coding conventions

- Use arrow functions only for all function declarations (components, hooks, utilities, handlers, and callbacks). Do not use `function` declarations.
- Prefer `const` for all variables; avoid `var`.
- Use named exports for utilities and components; use default exports for page and layout components where required by the framework.
- Keep components small and focused. Extract reusable logic into custom hooks or utilities.
- Use TypeScript for all source files. Avoid `any` unless absolutely necessary and documented.
- Place tests in `__tests__` directories colocated with the code they test (e.g., `src/components/__tests__/button.test.tsx`).
