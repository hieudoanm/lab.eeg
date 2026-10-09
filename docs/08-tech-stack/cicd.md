## 13. CI/CD

Use GitHub Actions.

The CI pipeline should run:

- Frontend dependency installation.
- Frontend linting.
- TypeScript checks.
- Frontend unit tests.
- Backend dependency installation.
- Python linting and formatting checks.
- Python type checking.
- Backend unit tests.
- Database integration tests.
- Migration validation.
- Build verification.
- Relevant security and dependency checks.

The deployment pipeline should:

1. Build immutable artefacts.
2. Run automated checks.
3. Deploy to a non-production environment where configured.
4. Apply migrations in a controlled manner.
5. Run smoke tests.
6. Promote to production only when the configured checks succeed.

Never deploy production changes directly from an unreviewed local working tree.

