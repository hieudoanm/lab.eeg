## 17. Risk Register

| Risk                             | Impact                          | Mitigation                                                  |
| -------------------------------- | ------------------------------- | ----------------------------------------------------------- |
| Scope expansion                  | Delayed delivery                | Keep the first analysis deliberately small                  |
| Tenant isolation bug             | Data exposure                   | Centralised permission checks and integration tests         |
| Unsafe file parsing              | Security and reliability issues | Strict format validation and resource limits                |
| Duplicate job delivery           | Inconsistent results            | Idempotent workers and database state checks                |
| Lost queue messages              | Stuck jobs                      | Transactional outbox and recovery mechanisms                |
| Excessive cloud cost             | Unexpected expense              | Budgets, resource limits, alerts, and teardown instructions |
| Scientific misinterpretation     | Misleading claims               | Clear units, provenance, and limitations                    |
| Complex infrastructure too early | Slow development                | Prove the local vertical slice first                        |
| Inconsistent API types           | Integration defects             | Generate types from OpenAPI where practical                 |
| Insufficient test realism        | Hidden database bugs            | Use real PostgreSQL integration tests                       |

