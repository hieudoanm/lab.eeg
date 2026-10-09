## 4. Local Development

### Prerequisites

Install supported versions of:

- Node.js.
- pnpm.
- Python.
- The selected Python dependency manager.
- Docker and Docker Compose.
- Git.
- Terraform, if working on infrastructure.

Run `pnpm install` from the repository root to install JavaScript dependencies and set up Husky Git hooks. Before each commit, lint-staged runs Ruff on staged backend Python files and ESLint/Prettier on staged frontend source files; these local checks do not replace CI.

Pin or document supported versions. Do not assume that the newest release of every tool is compatible with the repository.

### Suggested local services

Run these services with Docker Compose (`docker compose up --build`):

- Next.js frontend (`localhost:3000`).
- FastAPI API (`localhost:8000`).
- PostgreSQL and Redis.
- Celery worker and outbox dispatcher.
- A one-shot migration service that completes before the API and worker start.

Compose persists PostgreSQL data and local uploads in named volumes. Use
`docker compose down` to stop the stack without deleting those volumes, or
`docker compose down --volumes` to remove the development data. The frontend's
browser API URL can be changed at build time with
`NEXT_PUBLIC_API_BASE_URL=http://localhost:8000/api/v1`. Server-side frontend
requests use `API_INTERNAL_BASE_URL`, which Compose sets to the backend service
address.

The frontend Docker build uses the repository-level pnpm workspace lockfile and
Next.js standalone output. For running application processes directly on the
host, use the package-specific commands below instead.

### Example `.env.example`

```dotenv
# Frontend
NEXT_PUBLIC_API_BASE_URL=http://localhost:8000/api/v1

# API
APP_ENV=development
DATABASE_URL=postgresql://eeg-lab:eeg-lab@localhost:5432/eeg-lab
REDIS_URL=redis://localhost:6379/0
LOG_LEVEL=DEBUG

# Storage
STORAGE_BACKEND=local
LOCAL_STORAGE_PATH=./.local-data/uploads

# Authentication
AUTH_MODE=development
AUTH_TOKEN_SECRET=insecure-development-secret-change-me
AUTH_JWT_ALGORITHMS=HS256
AUTH_TOKEN_ISSUER=eeg-lab
AUTH_TOKEN_AUDIENCE=eeg-lab-api
AUTH_COOKIE_NAME=eeg-lab_session
AUTH_COOKIE_SAMESITE=lax
DEV_ORGANISATION_NAME=Development Organisation
```

These values are examples for local development only. Production must use managed credentials and secure configuration. When `APP_ENV=production`, the API refuses to start with `AUTH_MODE=development`, a default or short `AUTH_TOKEN_SECRET`, or an asymmetric `AUTH_JWT_ALGORITHMS` value without `AUTH_PUBLIC_KEY`.

If the selected database driver uses a different connection-string format or separate connection settings, document and implement that format consistently.

### Local setup workflow

The exact commands depend on the package scripts and dependency manager selected for the repository. Document the final commands in the README and Makefile.

The setup process should perform these steps:

1. Install frontend dependencies.
2. Install backend and worker dependencies.
3. Start PostgreSQL and Redis.
4. Apply database migrations.
5. Generate synthetic EEG fixtures.
6. Start the API.
7. Start the worker.
8. Start the frontend.
9. Run the smoke tests.

### Local development acceptance criteria

- A new developer can reproduce the environment.
- No production credentials are required.
- Migrations work against a fresh database.
- The frontend can communicate with the API.
- The worker can process a submitted analysis.
- Synthetic uploads can be validated.
- The full analysis workflow can be demonstrated locally.

