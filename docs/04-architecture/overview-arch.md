## 2. Architectural Overview

EEG Lab uses a modular monolith for its application API, with a separately deployed worker for background processing.

The frontend, API, worker, and supporting services have distinct responsibilities. The initial application should not be decomposed into many independent microservices.

### High-level architecture

```mermaid
flowchart TB
    U[Researcher / Viewer / Owner]

    subgraph Client["Client Layer"]
        WEB[Next.js + React + TypeScript]
        CHART[Plotly Signal Explorer]
    end

    subgraph Application["Application Layer"]
        API[FastAPI REST API]
        AUTH[Authentication and Authorisation]
        SVC[Application Services]
        REPO[Explicit SQL Repositories]
    end

    subgraph Processing["Background Processing"]
        REDIS[Redis Broker]
        WORKER[Celery Worker]
        NUMPY[NumPy Analysis]
    end

    subgraph Persistence["Persistence Layer"]
        PG[(PostgreSQL)]
        GCS[(Private Object Storage)]
    end

    U --> WEB
    WEB --> CHART
    WEB --> API
    API --> AUTH
    API --> SVC
    SVC --> REPO
    REPO --> PG
    API --> GCS
    SVC --> REDIS
    REDIS --> WORKER
    WORKER --> PG
    WORKER --> GCS
    WORKER --> NUMPY
```

The diagram shows logical responsibilities. The precise deployment topology may differ between local development and production.

The component architecture and detailed data-flow diagrams for system
boundaries, dataset upload, analysis processing, and CI/CD are indexed in
[`diagrams/`](diagrams/README.md).

