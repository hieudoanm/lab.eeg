# EEG Lab — Requirements

## 1. Purpose

This document defines the functional requirements, non-functional requirements, technical constraints, and acceptance criteria for EEG Lab.

All implementation decisions must align with these requirements and the project's `01-overview.md`, `04-features.md`, and `05-architecture.md`.

Requirements should be implemented incrementally according to `11-implementation-plan.md`. Do not attempt to implement the entire platform in a single pass.

## 2. Scope

The MVP must provide:

- User authentication and organisation membership.
- Study management.
- EEG dataset upload, validation, and provenance tracking.
- Interactive scientific signal visualisation.
- Asynchronous descriptive analysis.
- Analysis job monitoring and result inspection.
- Role-based access control and tenant isolation.
- Automated tests and continuous integration.
- Reproducible local development and documented cloud deployment.

Advanced machine learning, real-time acquisition, clinical interpretation, and commercial billing are outside the initial scope.

