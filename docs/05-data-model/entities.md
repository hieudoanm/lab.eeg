## 3. Entity Overview

The initial schema contains these principal entities:

| Entity             | Purpose                                               |
| ------------------ | ----------------------------------------------------- |
| `users`            | Application user identities                           |
| `organisations`    | Research workspaces and tenant boundaries             |
| `memberships`      | User roles within organisations                       |
| `studies`          | Research projects within an organisation              |
| `datasets`         | Recording metadata and object-storage references      |
| `upload_sessions`  | Upload lifecycle and finalisation state               |
| `analysis_jobs`    | Durable asynchronous job state                        |
| `analysis_results` | Persisted scientific output and provenance            |
| `outbox_events`    | Reliable dispatch of background work                  |
| `idempotency_keys` | Duplicate-request protection                          |
| `audit_events`     | Security-relevant and operationally important actions |

An authentication provider may own the authoritative identity record. The local `users` table should map to that identity rather than duplicate credentials unnecessarily.

