# EEG Lab Architecture

This logical architecture shows the implemented application boundaries and
runtime dependencies. The API is a modular monolith; analysis and outbox
dispatch run as separate processes. PostgreSQL owns durable state, Redis
transports task messages, and private object storage holds recording bytes.

```mermaid
flowchart TB
    Researcher[Researcher / viewer / owner]

    subgraph Client["Client"]
        Browser[Web browser]
        Frontend["Next.js application\nReact + TypeScript\ntRPC BFF (route handler + server caller)"]
        UploadProxy["Authenticated upload proxy\nstreaming upload body"]
        Charts["Signal explorer\nPlotly.js"]
    end

    subgraph Application["Application boundary"]
        API["FastAPI API"]
        Auth["Authentication and\norganisation authorisation"]
        Services["Application services"]
        Repositories["Explicit SQL repositories"]
        Validation["Recording validation\nNumPy scientific functions"]
    end

    subgraph Background["Background processes"]
        Dispatcher["Outbox dispatcher\nstale-job recovery"]
        Worker["Celery worker"]
        Analysis["Analysis execution\nclaim, compute, persist"]
    end

    subgraph Infrastructure["State and storage"]
        Postgres[("PostgreSQL\nmetadata, tenant relationships,\njobs, outbox, audit, results")]
        Redis[("Redis\nCelery task broker")]
        Objects[("Private object storage\nsource recordings")]
    end

    Researcher --> Browser
    Browser <-->|tRPC procedures| Frontend
    Frontend <-->|Authenticated API requests| API
    Frontend --> Charts
    Frontend -->|Recording upload| UploadProxy
    UploadProxy -->|Authenticated PUT stream| API

    API --> Auth
    Auth --> Services
    Services --> Repositories
    Repositories <-->|Parameterised, tenant-scoped SQL| Postgres
    Services -->|Create upload session and save bytes| Objects
    Services --> Validation
    Validation -->|Validated recording metadata| Services
    Services -->|Create job + outbox event atomically| Postgres

    Dispatcher <-->|Claim event, mark published,\nrecover stale jobs| Postgres
    Dispatcher -->|Publish job identifier| Redis
    Redis -->|At-least-once task message| Worker
    Worker --> Analysis
    Analysis <-->|Claim job, store result/status| Postgres
    Analysis -->|Read recording by object key| Objects
    Analysis -->|Verify input and compute statistics| Validation

    Postgres -->|Authoritative job status and results| API
    API -->|Scoped resource response| Frontend
```

## Boundary notes

- The browser can present organisation and resource identifiers, but the API
  derives the authenticated identity and verifies membership and role before
  authorising access.
- The browser calls tRPC procedures on the Next.js BFF, which reads the HttpOnly
  session cookie, resolves the active organisation, and forwards authenticated
  REST requests; it never bypasses API authorisation.
- The frontend upload proxy forwards the browser's authenticated session to
  the backend-mediated content endpoint. File validation and storage writes
  remain server-side.
- The dispatcher publishes job identifiers; it does not put the recording or
  analysis result in Redis.
- The worker reloads job and dataset state from PostgreSQL and reads the source
  recording from object storage. Duplicate messages do not make Redis the
  source of truth.
- The application can use local storage in development or the configured
  private object-storage adapter. The diagram does not imply a specific cloud
  deployment topology.
