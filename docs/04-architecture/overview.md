# EEG Lab — System Architecture

## 1. Purpose

This document describes the system architecture, component responsibilities, data flows, trust boundaries, and failure-recovery strategies for EEG Lab.

The architecture must support a secure neuroscience research workflow while remaining small enough for an independently developed portfolio project.

The primary architectural goals are:

- Clear separation of responsibilities.
- Strong organisation-level data isolation.
- Reliable asynchronous analysis.
- Reproducible scientific computation.
- Safe handling of large EEG recordings.
- Explicit and testable database access.
- Observable failures and recoverable operations.
- Reproducible local development and cloud deployment.

