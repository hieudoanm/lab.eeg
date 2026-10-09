## 19. Continuous Integration Gates

Every pull request should pass the checks relevant to the changed code.

### Required gates

- Formatting.
- Linting.
- Type checking.
- Frontend unit tests.
- Backend unit tests.
- Scientific correctness tests.
- Database integration tests.
- API and authorisation tests.
- Worker lifecycle tests where relevant.
- Production frontend build.
- Backend import and startup checks.
- SQL migration checks.

End-to-end tests should run on pull requests when practical and must run before a release or production deployment.

Do not weaken critical security or scientific correctness gates solely to make a pipeline green.

For fast local feedback, the Husky pre-commit hook uses lint-staged to run Ruff on staged Python files and ESLint/Prettier on staged frontend source files. These hooks do not replace any required CI gate.

