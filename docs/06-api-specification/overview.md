# EEG Lab — API Specification

## 1. Purpose

This document defines the initial REST API for EEG Lab, an independent research-oriented neuroscience application for managing EEG datasets and running reproducible descriptive analyses.

The API is implemented with Python, FastAPI, and Pydantic. PostgreSQL is the authoritative source for application metadata, job state, and analysis results. Scientific recordings are stored in private object storage.

The API must support:

- Organisation-scoped access control.
- Study and dataset management.
- Secure recording uploads.
- Dataset validation and metadata retrieval.
- Asynchronous analysis submission.
- Analysis status polling and result retrieval.
- Auditability and reproducibility.
- Consistent errors, pagination, and idempotency.

This is an initial implementation contract. Endpoints may be added as the product evolves, but existing contracts should not change without a deliberate versioning decision.

