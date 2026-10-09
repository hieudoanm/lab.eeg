# End-to-End Workflow Sequence

This sequence follows the implemented path from dataset registration through
upload validation and asynchronous analysis. Protected API requests authenticate
the user and verify organisation membership and role before accessing
organisation-scoped records. Upload validation is synchronous; analysis
computation is asynchronous.

`Web` denotes the Next.js application as a whole: the browser calls tRPC
procedures, which the tRPC BFF executes on the server, reads the HttpOnly session
cookie, resolves the active organisation, and forwards the REST request to
FastAPI. The browser never calls FastAPI directly.

```mermaid
sequenceDiagram
    autonumber
    actor Researcher
    participant Web as Next.js app (browser + tRPC BFF)
    participant Proxy as Upload proxy
    participant API as FastAPI
    participant DB as PostgreSQL
    participant Storage as Private object storage
    participant Dispatcher as Outbox dispatcher
    participant Redis as Redis broker
    participant Worker as Celery worker

    Researcher->>Web: Enter dataset metadata and select recording
    Web->>API: POST /organisations/{org}/studies/{study}/datasets
    Note over API: Authenticate and authorize organisation write access
    API->>DB: Insert pending dataset
    DB-->>API: Dataset ID
    API-->>Web: 201 Created with dataset

    Web->>API: POST /organisations/{org}/datasets/{dataset}/upload-sessions
    Note over API: Verify dataset belongs to the authorized organisation
    API->>DB: Insert upload session, expected size/checksum, and manifest
    DB-->>API: Upload session ID and generated object key
    API-->>Web: 201 Created with relative upload URL

    Web->>Proxy: PUT /api/uploads/content with recording bytes
    Proxy->>API: Authenticated PUT /organisations/{org}/datasets/{dataset}/upload-sessions/{session}/content
    Note over API: Authenticate, check write role, size limit, and session ownership
    API->>Storage: Write bytes to generated private object key
    Storage-->>API: Write complete
    API->>DB: Mark upload session uploaded
    DB-->>API: Session updated
    API-->>Proxy: 204 No Content
    Proxy-->>Web: 204 No Content

    Web->>API: POST /organisations/{org}/datasets/{dataset}/upload-sessions/{session}/finalise
    Note over API: Claim finalisation; verify size and checksum; parse and validate NPY against manifest
    API->>Storage: Read uploaded recording
    Storage-->>API: Recording bytes
    alt Recording valid
        API->>DB: Persist metadata and set dataset ready, session completed
        DB-->>API: Updated dataset and session
        API-->>Web: Finalisation response with ready dataset
    else Recording invalid
        API->>DB: Persist dataset rejected and session failed
        DB-->>API: Updated dataset and session
        API-->>Web: Safe validation error
        Web-->>Researcher: Display validation failure
    end

    opt Dataset is ready
        Researcher->>Web: Submit descriptive analysis
        Web->>API: POST /organisations/{org}/datasets/{dataset}/analyses with Idempotency-Key
        Note over API: Authenticate, authorize, verify ready dataset, and hash request
        API->>DB: Begin transaction
        API->>DB: Reserve idempotency key; create queued job and outbox event; record audit event
        DB-->>API: Commit transaction
        API-->>Web: 202 Accepted with queued job

        loop Dispatcher runs periodically
            Dispatcher->>DB: Claim pending outbox events using row locks
            DB-->>Dispatcher: Event payload containing job ID
            Dispatcher->>Redis: Publish Celery task with job ID
            Redis-->>Dispatcher: Publish acknowledgement
            Dispatcher->>DB: Mark outbox event published
        end

        Redis-->>Worker: Deliver analysis task with job ID
        Worker->>DB: Load job and atomically claim queued job
        DB-->>Worker: Job and dataset metadata
        Worker->>Storage: Read dataset recording
        Storage-->>Worker: Recording bytes
        Note over Worker: Re-validate input and compute descriptive statistics
        alt Analysis succeeds
            Worker->>DB: Persist result and mark job succeeded in one transaction
            DB-->>Worker: Commit result and job state
        else Retryable failure with attempts remaining
            Worker->>DB: Requeue job and create a new outbox event
            DB-->>Worker: Commit retry state
        else Non-retryable or attempts exhausted
            Worker->>DB: Mark job failed with safe error details
            DB-->>Worker: Commit terminal state
        end

        loop Frontend polls job status
            Web->>API: GET /organisations/{org}/analyses/{job}
            Note over API: Re-authenticate and scope the lookup to the organisation
            API->>DB: Read current job state
            DB-->>API: Job status
            API-->>Web: Current status
        end

        opt Job succeeded
            Web->>API: GET /organisations/{org}/analyses/{job}/results
            API->>DB: Read tenant-scoped persisted result
            DB-->>API: Result and provenance
            API-->>Web: Analysis result
            Web-->>Researcher: Display descriptive statistics
        end
    end
```

An identical analysis request using the same idempotency key replays the
original job instead of creating a second one. Redis delivers task notifications
only; PostgreSQL remains authoritative for job state and results.
