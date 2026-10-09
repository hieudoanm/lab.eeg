# EEG Lab — Testing Strategy

## 1. Purpose

This document defines the testing approach for EEG Lab, an independent research-oriented neuroscience SaaS application.

EEG Lab combines a web frontend, API, relational database, object storage, asynchronous workers, and scientific computations. Testing must verify not only that individual functions work, but also that these components behave correctly together under normal operation, invalid input, concurrent requests, and failure conditions.

The primary goals are to:

- Protect scientific calculation correctness.
- Prevent cross-organisation data access.
- Verify database integrity and transaction behaviour.
- Validate untrusted EEG uploads safely.
- Ensure reliable asynchronous processing.
- Maintain API and frontend contracts.
- Detect regressions through continuous integration.
- Demonstrate professional software engineering practices.

