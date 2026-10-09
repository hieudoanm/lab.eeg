# Architecture and Data-Flow Diagrams

This document contains Mermaid diagrams for EEG Lab's architecture. Render with any Markdown viewer that supports Mermaid (GitHub, GitLab, VS Code with extension, etc.).

## 1. High-Level System Architecture

```mermaid
graph TB
    subgraph "Client Layer"
        Browser[Browser\nNext.js + React + TypeScript]
        Plotly[Plotly.js Charts]
    end

    subgraph "Frontend Server Layer"
        BFF[tRPC Backend-for-Frontend\nNext.js Server]
    end

    subgraph "API Layer"
        API[FastAPI REST API\n/api/v1]
        Health[Health Endpoints\n/live, /ready]
        Auth[Auth\n/dev-token]
    end

    subgraph "Data Layer"
        PG[(PostgreSQL\nPrimary)]
        Redis[(Redis\nCelery Broker)]
    end

    subgraph "Storage Layer"
        LocalFS[Local Filesystem\n./local-data/uploads]
        GCS[GCS Bucket\nFuture]
    end

    subgraph "Workers"
        Worker[Celery Worker\nAnalysis Tasks]
        Dispatcher[Outbox Dispatcher\nFOR UPDATE SKIP LOCKED]
        Cleanup[Cleanup Worker\nAbandoned Uploads]
    end

    Browser -->|tRPC| BFF
    BFF -->|HTTPS bearer| API
    API -->|SQL| PG
    API -->|Publish Task| Redis
    API -->|Write Objects| LocalFS
    Redis -->|Consume Task| Worker
    Redis -->|Consume Event| Dispatcher
    Worker -->|Read Objects| LocalFS
    Worker -->|Write Results| PG
    Dispatcher -->|Mark Published| PG
    Cleanup -->|Expire Sessions| PG
    Cleanup -->|Delete Objects| LocalFS

    classDef client fill:#e3f2fd,stroke:#1976d2
    classDef bff fill:#e1f5fe,stroke:#0277bd
    classDef api fill:#f3e5f5,stroke:#7b1fa2
    classDef data fill:#e8f5e9,stroke:#388e3c
    classDef storage fill:#fff3e0,stroke:#f57c00
    classDef worker fill:#fce4ec,stroke:#c2185b

    class Browser,Plotly client
    class BFF bff
    class API,Health,Auth api
    class PG,Redis data
    class LocalFS,GCS storage
    class Worker,Dispatcher,Cleanup worker
```

## 2. Request Lifecycle (Analysis Submission)

The browser never calls FastAPI directly: it calls tRPC procedures on the Next.js BFF, which forwards authenticated REST requests.

```mermaid
sequenceDiagram
    participant User as Researcher
    participant Browser as Browser (React)
    participant BFF as tRPC BFF (Next.js)
    participant API as FastAPI
    participant PG as PostgreSQL
    participant Redis as Redis
    participant Worker as Celery Worker
    participant Storage as Object Storage

    User->>Browser: Click "Submit Analysis"
    Browser->>BFF: analyses.submit\n{dataset_id, parameters, idempotency_key}
    BFF->>API: POST /datasets/{id}/analyses\nIdempotency-Key: abc123
    API->>PG: BEGIN TRANSACTION
    API->>PG: SELECT dataset (tenant check)
    API->>PG: INSERT idempotency_keys\nON CONFLICT DO NOTHING
    API->>PG: INSERT analysis_jobs (queued)
    API->>PG: INSERT outbox_events (analysis.requested)
    API->>PG: COMMIT
    API-->>BFF: 202 Accepted\n{job_id, status: "queued"}
    BFF-->>Browser: {job_id, status: "queued"}
    Browser-->>User: Show "Analysis submitted"

    par Dispatcher Loop
        Dispatcher->>PG: SELECT ... FOR UPDATE SKIP LOCKED\nWHERE published_at IS NULL
        Dispatcher->>Redis: send_task("eeg-lab.run_analysis", payload)
        Dispatcher->>PG: UPDATE outbox_events SET published_at=now()
    end

    Worker->>Redis: Reserve task
    Worker->>PG: UPDATE analysis_jobs SET status=running,\n  attempt_count+=1, lease_expires_at=...
    Worker->>Storage: Read recording (.npy)
    Worker->>Worker: Validate checksum, manifest
    Worker->>Worker: Compute statistics (NumPy)
    Worker->>PG: BEGIN TRANSACTION
    Worker->>PG: INSERT analysis_results\n  ON CONFLICT DO NOTHING
    Worker->>PG: UPDATE analysis_jobs SET status=succeeded
    Worker->>PG: INSERT audit_events (analysis.completed)
    Worker->>PG: COMMIT
    Worker-->>Redis: Task complete

    User->>Browser: Poll analysis status
    Browser->>BFF: analyses.byId / analyses.result
    BFF->>API: GET /analyses/{job_id} [+ /results]
    API->>PG: SELECT analysis_jobs / analysis_results
    API-->>BFF: {status, results, provenance}
    BFF-->>Browser: Render charts + statistics table
    Browser-->>User: See results
```

## 3. Upload & Validation Flow

Only recording metadata travels through the tRPC BFF; the raw array bytes stream through the `/api/uploads/content` route handler.

```mermaid
sequenceDiagram
    participant User as Researcher
    participant Browser as Browser (React)
    participant BFF as tRPC BFF (Next.js)
    participant Route as Upload Route Handler
    participant API as FastAPI
    participant PG as PostgreSQL
    participant Storage as Object Storage

    User->>Browser: Select .npy file + metadata
    Browser->>BFF: uploads.createSession\n{file_name, file_size, manifest}
    BFF->>API: POST /datasets/{id}/upload-sessions
    API->>PG: INSERT upload_sessions (created)
    API-->>BFF: {session_id, upload_url}
    BFF-->>Browser: {session_id}
    Browser->>Route: PUT /api/uploads/content?session_id=...
    Route->>API: PUT /upload-sessions/{id}/content\nBody: .npy bytes
    API->>Storage: Write bytes to key
    API-->>Route: 204 No Content
    Route-->>Browser: 204 No Content
    Browser->>BFF: uploads.finalise
    BFF->>API: POST /upload-sessions/{id}/finalise
    API->>Storage: Verify size + SHA-256
    API->>Storage: Load .npy (mmap, no pickle)
    API->>API: Validate shape, dtype, channels, sampling freq
    API->>PG: UPDATE datasets SET status=ready,\n  storage_key, checksum, metadata
    API->>PG: UPDATE upload_sessions SET status=completed
    API-->>BFF: 200 OK\n{dataset: {status: "ready", ...}}
    BFF-->>Browser: {dataset: {status: "ready", ...}}
```

## 4. Tenant Isolation Model

```mermaid
graph TD
    subgraph "Organisation A"
        UserA1[User: alice\nRole: owner]
        UserA2[User: bob\nRole: researcher]
        StudyA1[Study: "Sleep Study"]
        DatasetA1[Dataset: "night1.npy"]
        JobA1[Analysis Job\nstatus: succeeded]
        ResultA1[Analysis Result\nprovenance + channels]
    end

    subgraph "Organisation B"
        UserB1[User: charlie\nRole: viewer]
        StudyB1[Study: "Seizure Study"]
        DatasetB1[Dataset: "patient2.npy"]
    end

    UserA1 -->|owns| StudyA1
    UserA2 -->|member| StudyA1
    StudyA1 -->|contains| DatasetA1
    DatasetA1 -->|produced| JobA1
    JobA1 -->|generated| ResultA1

    UserB1 -->|member| StudyB1
    StudyB1 -->|contains| DatasetB1

    classDef orgA fill:#e3f2fd,stroke:#1976d2
    classDef orgB fill:#fff3e0,stroke:#f57c00
    classDef cross fill:#ffebee,stroke:#c62828,stroke-dasharray: 5 5

    class UserA1,UserA2,StudyA1,DatasetA1,JobA1,ResultA1 orgA
    class UserB1,StudyB1,DatasetB1 orgB

    Note right of UserA1: All queries filtered by\norganisation_id from JWT
    Note right of UserB1: Cannot see Org A resources\n(404 not 403)
    Note left of ResultA1: Provenance includes\nchecksum + algorithm version
```

## 5. Outbox Pattern (Reliable Event Dispatch)

```mermaid
stateDiagram-v2
    [*] --> Pending : INSERT outbox_events\n(published_at=NULL)
    Pending --> Claimed : Dispatcher\nFOR UPDATE SKIP LOCKED
    Claimed --> Published : Redis send_task OK\nUPDATE published_at=now()
    Claimed --> Failed : Redis send_task ERR\nattempt_count+=1\nnext_attempt_at=now+backoff
    Failed --> Claimed : next_attempt_at <= now
    Failed --> Exhausted : attempt_count >= max_attempts\n(10 default)\nnext_attempt_at=now+100y\nlast_error_code=exhausted
    Published --> [*]
    Exhausted --> [*] : Manual intervention\nor admin reprocess

    note right of Pending
        Atomic with job creation:
        - analysis_jobs (queued)
        - outbox_events (pending)
        - idempotency_keys (reserved)
        All in ONE transaction
    end note

    note right of Claimed
        Row lock held across
        Redis publish call.
        Acceptable because
        send_task is sub-ms.
    end note
```

## 6. Analysis Job State Machine

```mermaid
stateDiagram-v2
    [*] --> Queued : API submit\n(202 Accepted)
    Queued --> Running : Worker claim_job\n(attempt_count+=1, lease)
    Queued --> Cancelled : API cancel\n(cooperative, idempotent)
    Running --> Succeeded : Worker complete_job\n(result committed)
    Running --> Failed : Worker fail_job\n(error_code, message)
    Running --> Queued : Worker requeue_job\n(transient error + attempt<max)
    Running --> Queued : Recovery requeue_job\n(lease expired, attempt<max)
    Running --> Failed : Recovery fail_job\n(lease expired, attempt>=max\n  error_code=analysis_stale)
    Succeeded --> [*]
    Failed --> [*]
    Cancelled --> [*]

    note right of Running
        Lease = max(job_lease_seconds,\ncelery_task_time_limit+30s)
        Prevents premature reclaim.
    end note

    note right of Queued
        Idempotency key scope:\n(org_id, user_id,\noperation="analysis.submit", key)
    end note
```

## 7. Data Model Relationships

```mermaid
erDiagram
    ORGANISATIONS ||--o{ MEMBERSHIPS : has
    ORGANISATIONS ||--o{ STUDIES : owns
    ORGANISATIONS ||--o{ DATASETS : owns
    ORGANISATIONS ||--o{ ANALYSIS_JOBS : submits
    ORGANISATIONS ||--o{ ANALYSIS_RESULTS : produces
    ORGANISATIONS ||--o{ OUTBOX_EVENTS : emits
    ORGANISATIONS ||--o{ IDEMPOTENCY_KEYS : reserves
    ORGANISATIONS ||--o{ AUDIT_EVENTS : records

    USERS ||--o{ MEMBERSHIPS : belongs
    USERS ||--o{ STUDIES : creates
    USERS ||--o{ DATASETS : uploads
    USERS ||--o{ ANALYSIS_JOBS : requests
    USERS ||--o{ AUDIT_EVENTS : acts

    STUDIES ||--o{ DATASETS : contains
    DATASETS ||--o{ UPLOAD_SESSIONS : receives
    DATASETS ||--o{ ANALYSIS_JOBS : analyses
    DATASETS ||--o{ ANALYSIS_RESULTS : results_for

    ANALYSIS_JOBS ||--|| ANALYSIS_RESULTS : produces
    ANALYSIS_JOBS ||--o{ OUTBOX_EVENTS : triggers
    ANALYSIS_JOBS ||--|| IDEMPOTENCY_KEYS : deduplicates

    UPLOAD_SESSIONS {
        uuid id PK
        uuid dataset_id FK
        enum status
        string upload_method
        timestamp expires_at
    }

    DATASETS {
        uuid id PK
        uuid organisation_id FK
        uuid study_id FK
        enum status
        enum format
        string storage_key
        string checksum_sha256
        int file_size_bytes
        float sampling_frequency_hz
        int channel_count
        int sample_count
        json channel_names
        json channel_units
    }

    ANALYSIS_JOBS {
        uuid id PK
        uuid organisation_id FK
        uuid dataset_id FK
        uuid requested_by FK
        enum analysis_type
        string algorithm_version
        json parameters
        enum status
        int attempt_count
        int max_attempts
        string error_code
        string error_message
        uuid idempotency_key
        timestamp lease_expires_at
    }

    ANALYSIS_RESULTS {
        uuid id PK
        uuid organisation_id FK
        uuid job_id FK UNIQUE
        uuid dataset_id FK
        string dataset_checksum_sha256
        string algorithm_version
        string result_schema_version
        json results
        json provenance
    }

    OUTBOX_EVENTS {
        uuid id PK
        uuid organisation_id FK
        string event_type
        string aggregate_type
        uuid aggregate_id
        json payload
        timestamp published_at
        int attempt_count
        timestamp next_attempt_at
        string last_error_code
    }

    IDEMPOTENCY_KEYS {
        uuid id PK
        uuid organisation_id FK
        uuid user_id FK
        string operation
        string key
        string request_hash
        int response_status
        json response_body
        uuid resource_id
        timestamp expires_at
    }
```

## 8. Deployment Architecture (Target - Phase 10)

```mermaid
graph TB
    subgraph "Internet"
        Users[Users]
    end

    subgraph "GCP Project"
        subgraph "VPC - Private"
            subgraph "Cloud Run - API"
                API1[API Instance 1]
                API2[API Instance 2]
                API3[API Instance N]
            end

            subgraph "Cloud Run - Worker"
                Worker1[Worker Instance 1]
                Worker2[Worker Instance 2]
            end

            subgraph "Cloud Run - Frontend"
                FE1[Frontend Instance 1]
                FE2[Frontend Instance N]
            end

            subgraph "Cloud SQL - PostgreSQL"
                PGPrimary[(Primary)]
                PGReplica[(Read Replica)]
            end

            subgraph "Memorystore - Redis"
                RedisPrimary[(Primary)]
                RedisReplica[(Replica)]
            end

            subgraph "Cloud Storage"
                Bucket[(eeg-lab-uploads\nVersioned, Private)]
            end

            subgraph "Secret Manager"
                Secrets[API Keys, DB Password,\nJWT Secret, Redis Auth]
            end
        end

        subgraph "Cloud Run - Dispatcher (Cloud Scheduler)"
            Scheduler[Cloud Scheduler\nEvery 30s] --> Dispatcher[Dispatcher Job]
        end

        subgraph "Monitoring"
            Logging[Cloud Logging]
            Monitoring[Cloud Monitoring\n+ Alerting]
            Trace[Cloud Trace]
        end
    end

    Users -->|HTTPS| FE1
    Users -->|HTTPS| API1
    FE1 -->|API Calls| API1
    API1 -->|SQL| PGPrimary
    API1 -->|Tasks| RedisPrimary
    API1 -->|Objects| Bucket
    API1 -->|Secrets| Secrets
    Worker1 -->|SQL| PGPrimary
    Worker1 -->|Tasks| RedisPrimary
    Worker1 -->|Objects| Bucket
    Dispatcher -->|SQL| PGPrimary
    Dispatcher -->|Tasks| RedisPrimary

    API1 -.->|Logs/Metrics| Logging
    API1 -.->|Traces| Trace
    Worker1 -.->|Logs/Metrics| Logging
    Worker1 -.->|Traces| Trace

    classDef run fill:#e3f2fd,stroke:#1976d2
    classDef sql fill:#e8f5e9,stroke:#388e3c
    classDef redis fill:#fff3e0,stroke:#f57c00
    classDef storage fill:#fce4ec,stroke:#c2185b
    classDef secret fill:#f3e5f5,stroke:#7b1fa2
    classDef monitor fill:#f1f8e9,stroke:#689f38

    class API1,API2,API3,Worker1,Worker2,FE1,FE2,Dispatcher run
    class PGPrimary,PGReplica sql
    class RedisPrimary,RedisReplica redis
    class Bucket storage
    class Secrets secret
    class Logging,Monitoring,Trace monitor
```

---

_Diagrams rendered with Mermaid. For interactive viewing, open in GitHub/GitLab or VS Code with Mermaid extension._
