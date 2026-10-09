## 4. Entity Relationship Diagram

```mermaid id="d9n0fr"
erDiagram
    USERS ||--o{ MEMBERSHIPS : has
    ORGANISATIONS ||--o{ MEMBERSHIPS : contains
    ORGANISATIONS ||--o{ STUDIES : owns
    STUDIES ||--o{ DATASETS : contains
    DATASETS ||--o{ UPLOAD_SESSIONS : has
    DATASETS ||--o{ ANALYSIS_JOBS : analysed_by
    ANALYSIS_JOBS ||--o| ANALYSIS_RESULTS : produces
    ORGANISATIONS ||--o{ ANALYSIS_JOBS : scopes
    ORGANISATIONS ||--o{ AUDIT_EVENTS : records
    ORGANISATIONS ||--o{ OUTBOX_EVENTS : scopes
    ORGANISATIONS ||--o{ IDEMPOTENCY_KEYS : scopes
    USERS ||--o{ STUDIES : creates
    USERS ||--o{ DATASETS : uploads
    USERS ||--o{ ANALYSIS_JOBS : requests
    USERS ||--o{ AUDIT_EVENTS : performs
```

Some supporting relationships, such as references to a dataset version, may be implemented as additional columns or tables as the product evolves.

The current PostgreSQL schema is also available as a DBML diagram source at
[`database/eeg-lab.dbml`](database/eeg-lab.dbml). It reflects the versioned SQL
migrations under `packages/backend/migrations/versions/`.

