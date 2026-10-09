# Dataset Upload and Validation Data Flow

The current upload is mediated by the authenticated API. Registering the dataset
and finalising the upload are tRPC procedures executed by the Next.js BFF; the
recording bytes travel through the dedicated upload route, which forwards the
browser request to FastAPI. The API writes the recording to private storage.
Finalisation validates the recording synchronously.

```mermaid
flowchart TD
    User[Researcher]

    subgraph Client["Browser and frontend boundary"]
        Browser[Select recording and enter metadata]
        Web[Next.js application\ntRPC BFF + upload route]
        Proxy[Authenticated upload proxy\n/api/uploads/content]
    end

    subgraph App["Backend trust boundary"]
        API[FastAPI upload routes]
        Auth[Authenticate user\nverify organisation and write role]
        UploadService[Upload session and validation service]
        Limit[Enforce configured upload size limit]
        Validator[Check size, checksum, NPY shape,\nfinite values, and manifest]
    end

    subgraph Stores["Durable stores"]
        DB[(PostgreSQL\nDataset metadata, upload session,\nmanifest, status, audit event)]
        Objects[(Private object storage\nRecording bytes)]
    end

    User -->|Dataset fields and recording| Browser
    Browser --> Web
    Web -->|Register dataset and request session| API
    API --> Auth
    Auth --> UploadService
    UploadService -->|Persist pending dataset,\nsession, expected file metadata| DB
    API -->|Session ID and relative upload URL| Web
    Web -->|Recording bytes| Proxy
    Proxy -->|Authenticated PUT /content| API
    API --> Auth
    Auth --> UploadService
    UploadService --> Limit
    Limit -->|Accepted bytes| Objects
    UploadService -->|Mark session uploaded| DB
    Web -->|POST /finalise| API
    API --> Auth
    Auth --> UploadService
    UploadService -->|Claim session for finalisation| DB
    UploadService -->|Read uploaded recording| Objects
    Objects -->|Recording bytes| Validator
    Validator -->|Verified file size, checksum,\nrecording dimensions and metadata| UploadService
    UploadService -->|Persist ready dataset and completed session| DB
    UploadService -->|On invalid input, persist rejected dataset\nand failed session| DB
    API -->|Dataset and final status, or safe error| Web
    Web -->|Validation outcome| Browser
```

The upload session, generated object key, expected size/checksum, and supplied
manifest are persisted before bytes are accepted. Dataset status becomes
`ready` only after server-side validation succeeds; invalid recordings are
recorded as rejected. This path does not use Celery or Redis.
