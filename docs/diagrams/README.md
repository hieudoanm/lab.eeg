# EEG Lab Architecture and Data Flow Diagrams

These diagrams describe the implemented application architecture and data
flows. They complement the architecture notes in
[`../05-architecture.md`](../05-architecture.md).

| Diagram                                            | Scope                                                                      |
| -------------------------------------------------- | -------------------------------------------------------------------------- |
| [Architecture](architecture.md)                    | Logical application components, runtime processes, boundaries, and stores  |
| [System data flow](system-data-flow.md)            | User, frontend, API, persistence, storage, and worker boundaries           |
| [Dataset upload and validation](dataset-upload.md) | Backend-mediated upload session, recording validation, and dataset state   |
| [Asynchronous analysis](analysis-pipeline.md)      | Idempotent submission, transactional outbox, worker execution, and results |
| [End-to-end sequence](end-to-end-sequence.md)      | Dataset registration, upload, validation, analysis, and result retrieval   |
| [CI/CD pipeline](cicd-pipeline.md)                 | GitHub Actions validation and Cloud Build deployment configuration         |

Update these diagrams when the corresponding implementation or data contract
changes. The upload diagram describes the current backend-mediated upload; a
direct signed-URL upload is a possible future architecture and is not shown.
