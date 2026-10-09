# EEG Lab — Data Model and Database Schema

## 1. Purpose

This document defines EEG Lab's relational data model, entity relationships, PostgreSQL constraints, indexes, lifecycle rules, and data integrity requirements.

The data model must support:

- Multi-tenant research organisations.
- User membership and role-based access.
- Studies and EEG datasets.
- Dataset upload and validation.
- Asynchronous analysis jobs.
- Reproducible analysis results.
- Audit events.
- Idempotent operations.
- Reliable background-job dispatch.

PostgreSQL is the authoritative source for application metadata, job state, and persisted analysis results. Large scientific recordings are stored in private object storage.

