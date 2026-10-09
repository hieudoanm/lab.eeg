# Asynchronous Analysis Data Flow

Analysis submission records the job and its dispatch event atomically. The
dispatcher publishes only a job identifier; the worker reloads authoritative
job and dataset state from PostgreSQL before reading and processing the
recording.

```mermaid
flowchart TD
    User[Researcher]

    subgraph Client["Browser and frontend"]
        Web[Next.js application\ntRPC BFF]
    end

    subgraph API["API trust boundary"]
        APIProcess[FastAPI]
        Auth[Authenticate and verify organisation role]
        Submit[Validate request and ready dataset]
        Transaction[Single PostgreSQL transaction]
    end

    subgraph Durable["Durable state"]
        DB[(PostgreSQL\nIdempotency key, analysis job,\noutbox event, audit, result)]
        Objects[(Private object storage\nValidated recording)]
    end

    subgraph Queue["Background processing"]
        Dispatcher[Outbox dispatcher\nand stale-job recovery]
        Redis[(Redis task broker)]
        Worker[Celery worker]
        Compute[Re-validate recording\nand compute descriptive statistics]
    end

    User -->|Analysis request + Idempotency-Key| Web
    Web -->|POST analysis| APIProcess
    APIProcess --> Auth
    Auth --> Submit
    Submit -->|Ensure dataset belongs to tenant\nand is ready| DB
    Submit --> Transaction
    Transaction -->|Reserve idempotency key,\ncreate job and outbox event,\nrecord audit event| DB
    Transaction -->|Accepted job response (202)| APIProcess
    APIProcess --> Web

    Dispatcher -->|Claim pending events with row locks| DB
    Dispatcher -->|Publish job ID| Redis
    Dispatcher -->|Mark event published / retry failure| DB
    Redis -->|Task containing job ID| Worker
    Worker -->|Load and atomically claim queued job| DB
    Worker -->|Read dataset metadata and object key| DB
    Worker -->|Read recording bytes| Objects
    Worker --> Compute
    Compute -->|Result and provenance| Worker
    Worker -->|Persist result and succeed job atomically| DB
    Worker -->|Classified failure, requeue, or terminal failure| DB

    Web -->|Poll job status / request result| APIProcess
    APIProcess --> Auth
    Auth -->|Tenant-scoped job and result lookup| DB
    DB -->|Status or persisted result| APIProcess
    APIProcess --> Web
    Web -->|Progress or analysis result| User
```

The `Idempotency-Key` replays the original job for an identical request and
rejects reuse for a different request. Redis is at-least-once transport, not
authoritative state: duplicate task delivery is safe because the worker claims
only queued jobs and verifies state in PostgreSQL. Transient failures may
requeue within the bounded attempt limit; the dispatcher also recovers stale
jobs using their lease.
