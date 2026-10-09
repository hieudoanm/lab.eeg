## 12. Testing and Quality

### Frontend

Use:

- Vitest.
- React Testing Library.
- Playwright.
- TypeScript compiler checks.
- ESLint.

Test user-visible behaviour rather than implementation details.

### Backend

Use:

- pytest.
- HTTPX.
- Real PostgreSQL integration tests.
- Controlled worker and queue tests.
- Ruff.
- mypy.

Mock external services in unit tests where appropriate, but use real PostgreSQL for tests involving SQL semantics, constraints, transactions, or migrations.

### Scientific processing

Test:

- Known arrays with known expected statistics.
- Empty and malformed input.
- Invalid metadata.
- Non-finite values.
- Large input boundaries.
- Correct channel ordering.
- Numerical edge cases.
- Reproducibility for fixed input and parameters.

### End-to-end

Use Playwright for critical workflows:

1. Authenticate.
2. Create a study.
3. Upload a recording.
4. Inspect dataset metadata.
5. Explore a signal interval.
6. Submit an analysis.
7. Observe job completion.
8. Inspect the result.

