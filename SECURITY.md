# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| latest  | :white_check_mark: |

Only the latest `latest` release receives security updates. Older releases are not patched.

## Reporting a Vulnerability

Please report security vulnerabilities via **GitHub Security Advisories**:

1. Go to the [Security tab](https://github.com/hieudoanm/eeg-lab/security)
2. Click "Report a vulnerability"
3. Fill in the details

**Do not** open public issues for security vulnerabilities.

## Response Timeline

| Phase              | Target                                    |
| ------------------ | ----------------------------------------- |
| Acknowledgment     | 48 hours                                  |
| Initial assessment | 5 business days                           |
| Fix development    | 30 days (critical), 90 days (high/medium) |
| Release            | Next `latest` build after fix             |

## Security Architecture

### Authentication & Authorisation

- JWT-based sessions with HttpOnly cookies
- Organisation-scoped multi-tenancy (row-level security)
- Role-based access control (Owner/Admin/Member/Viewer)

### Data Protection

- TLS 1.3 in transit (enforced via HSTS)
- AES-256 at rest (database, object storage)
- No secrets in frontend bundles or logs
- PII minimisation — only email/name stored

### Infrastructure

- Dependency scanning: `pip-audit`, `npm audit`, Trivy (CI)
- Container base images pinned to distroless/slim variants
- Non-root containers, read-only rootfs where possible
- GitHub Actions: least-privilege permissions, pinned action SHAs

### API Security

- Input validation via Pydantic schemas
- Rate limiting on auth endpoints
- Idempotency keys for mutations
- Cursor pagination with max page size
- Error envelope never exposes stack traces or SQL

## Secure Development

- All PRs require CI pass (tests, lint, typecheck, security scan)
- No direct pushes to `master` — PRs only
- Signed commits required for maintainers
- Pre-commit hooks: ruff, prettier, secret detection

## Third-Party Components

Key dependencies monitored for CVEs:

- Python: FastAPI, SQLAlchemy, Pydantic, Celery, Redis
- Node: Next.js, React, tRPC, Tailwind, @tauri-apps/\*
- Rust: tauri, axum, sqlx, serde, tokio

## Disclosure

We follow coordinated disclosure. Credit given in release notes unless reporter requests anonymity.
