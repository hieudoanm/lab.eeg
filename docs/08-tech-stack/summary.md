## 3. Stack Summary

| Layer                  | Technology                                        | Responsibility                                         |
| ---------------------- | ------------------------------------------------- | ------------------------------------------------------ |
| Frontend               | React, TypeScript                                 | Interactive research interface                         |
| Frontend framework     | Next.js                                           | Routing, application structure, production builds      |
| Frontend desktop       | Tauri v2                                          | Native desktop app wrapper for WebView                 |
| Frontend data layer    | tRPC, React Query                                 | Typed Backend-for-Frontend and client state            |
| Styling                | Tailwind CSS v4 + daisyUI v5                      | Responsive styling, design tokens, and UI primitives   |
| UI components          | Accessible reusable React components              | Forms, tables, navigation, dialogs                     |
| Signal visualisation   | Plotly.js                                         | EEG time-series exploration                            |
| Backend                | Python, FastAPI                                   | REST API and business logic                            |
| Validation             | Pydantic                                          | Request, response, and configuration validation        |
| Database               | PostgreSQL                                        | Users, organisations, studies, datasets, jobs, results |
| Database driver        | Psycopg 3 asynchronous API                        | Explicit asynchronous PostgreSQL access                |
| Database migrations    | PostgreSQL `.sql` files and a Psycopg runner      | Versioned, reviewable schema changes                   |
| Background jobs        | Celery                                            | Asynchronous analysis processing                       |
| Queue and broker       | Redis                                             | Job delivery and coordination                          |
| Scientific computing   | NumPy                                             | Numerical operations on signal arrays                  |
| EEG file support       | NumPy `.npy` plus JSON manifest initially         | Recording storage and metadata                         |
| Object storage         | Google Cloud Storage                              | Private scientific files and artefacts                 |
| Containerisation       | Docker                                            | Reproducible application and worker environments       |
| Cloud infrastructure   | Google Cloud Platform                             | Hosting and managed infrastructure                     |
| Infrastructure as code | Terraform                                         | Reproducible infrastructure provisioning               |
| CI/CD                  | GitHub Actions                                    | Testing, quality checks, and deployment                |
| Frontend tests         | Vitest, React Testing Library                     | Component and interaction testing                      |
| Browser tests          | Playwright                                        | End-to-end workflows                                   |
| Backend tests          | pytest, HTTPX                                     | Unit and API integration testing                       |
| Python quality         | Ruff, mypy                                        | Linting, formatting, and static type checking          |
| TypeScript quality     | ESLint, TypeScript compiler                       | Linting and type checking                              |
| Observability          | Structured logs and Google Cloud monitoring tools | Operational diagnostics                                |

Versions should be pinned through the project's dependency-management files and updated deliberately.

