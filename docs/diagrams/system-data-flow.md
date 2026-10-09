# System Data Flow

This Level 0/1 data-flow view identifies the main actors, processes, data
stores, and external transport. Organisation membership and resource access
are verified by the API; the browser is not an authority for tenant access. The
browser calls tRPC procedures on the Next.js BFF, which forwards authenticated
REST requests to FastAPI.

```mermaid
flowchart LR
    User[Researcher / viewer / owner]

    subgraph Client["Client boundary"]
        Browser[Browser]
        Web[Next.js web application\ntRPC BFF]
    end

    subgraph App["Application trust boundary"]
        API[FastAPI API]
        Identity[Authenticate and verify organisation membership]
        Services[Application services]
        Scientific[Recording validation and NumPy analysis]
        Dispatcher[Outbox dispatcher]
        Worker[Celery analysis worker]
    end

    subgraph Stores["Persistence and transport"]
        DB[(PostgreSQL\nUsers, memberships, studies,\ndatasets, jobs, outbox, results)]
        Objects[(Private object storage\nRecording files)]
        Redis[(Redis\nTask transport only)]
    end

    User -->|Credentials, metadata, upload,\nanalysis requests| Browser
    Browser -->|UI and session requests| Web
    Web -->|Authenticated HTTPS API requests| API
    API --> Identity
    Identity -->|Verified user and organisation context| Services
    Services -->|Scoped SQL reads and writes| DB
    Services -->|Write/read recording bytes| Objects
    Services -->|Persist job and outbox event\nin one transaction| DB
    Dispatcher -->|Claim pending events| DB
    Dispatcher -->|Publish job identifier| Redis
    Redis -->|Analysis task identifier| Worker
    Worker -->|Claim job, persist status/result| DB
    Worker -->|Read source recording| Objects
    Worker --> Scientific
    Scientific -->|Validated input and statistics| Worker
    DB -->|Job status and persisted result| API
    API -->|Authorised response| Web
    Web -->|Status and results| Browser
```

Redis carries task notifications but is not the source of truth for job state.
PostgreSQL stores the authoritative job lifecycle; the recording stays in
private storage and is referenced by object key and validated metadata.
