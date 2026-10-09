## 4. Testing Tools

### Frontend

| Tool                  | Purpose                        |
| --------------------- | ------------------------------ |
| Vitest                | Unit and integration tests     |
| React Testing Library | Component behaviour            |
| Playwright            | Browser-based end-to-end tests |
| TypeScript            | Static type checking           |
| ESLint                | Code-quality checks            |

### Backend and worker

| Tool                                        | Purpose                                           |
| ------------------------------------------- | ------------------------------------------------- |
| pytest                                      | Test runner                                       |
| HTTPX                                       | API testing                                       |
| Pydantic                                    | Request and result validation                     |
| Psycopg 3                                   | Database integration                              |
| Psycopg-based SQL migration runner          | Migration validation                              |
| NumPy                                       | Numerical test fixtures and expected calculations |
| Testcontainers or an equivalent environment | Disposable PostgreSQL integration tests           |

### Infrastructure

- Terraform formatting and validation.
- Static infrastructure security checks.
- Container image scanning.
- Deployment smoke tests.

Use the smallest set of tools that provides the required coverage.

