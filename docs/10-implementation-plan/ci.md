## 15. Continuous Integration

Every pull request should run the checks relevant to the changed code.

### Frontend

- Formatting.
- Linting.
- Type checking.
- Unit tests.
- Production build.
- Selected component or browser tests.

### Backend

- Formatting.
- Linting.
- Type checking where configured.
- Unit tests.
- API tests.
- PostgreSQL integration tests.
- Migration checks.

### Worker

- Formatting and linting.
- Unit tests.
- Analysis correctness tests.
- Job lifecycle tests.
- Retry and duplicate-delivery tests.

### Infrastructure

- Terraform formatting.
- Terraform validation.
- Static security checks.
- Plan review for deployment changes.

Build and test containers using pinned or otherwise controlled dependency versions.

