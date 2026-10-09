## 2. Testing Principles

1. Test observable behaviour rather than implementation details.
2. Test critical business rules at the appropriate system boundary.
3. Use deterministic fixtures for scientific calculations.
4. Use real PostgreSQL for database-specific behaviour.
5. Test authorisation negatively, not only with authorised users.
6. Treat worker execution as potentially duplicated.
7. Test failure recovery as well as successful execution.
8. Keep unit tests fast and integration tests representative.
9. Avoid relying on live cloud resources for every test.
10. Make failures reproducible and actionable.
11. Keep test data synthetic unless a separate approved test-data process exists.
12. Do not equate high code coverage with correctness.

