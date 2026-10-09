## 10. CI/CD with GitHub Actions

Use GitHub Actions for validation and deployment.

### Pull request pipeline

Run:

- Frontend linting.
- Frontend type checking.
- Frontend tests.
- Backend linting and tests.
- Scientific correctness tests.
- PostgreSQL integration tests.
- Worker tests.
- Production builds.
- Migration checks.
- Terraform formatting and validation.
- Security checks where configured.

### Deployment pipeline

A production workflow should:

1. Trigger only from approved branches or releases.
2. Run required checks.
3. Build immutable images.
4. Publish images to a container registry.
5. Authenticate using workload identity federation where supported.
6. Apply approved infrastructure changes when needed.
7. Run database migrations safely.
8. Deploy the API, worker, and frontend.
9. Execute smoke tests.
10. Report deployment status.

Prefer immutable image tags based on commit SHA or image digest. Do not rely only on a mutable `latest` tag.

### Deployment permissions

- Use protected production environments.
- Restrict who can approve releases.
- Keep deployment permissions minimal.
- Separate pull-request validation from production credentials.
- Do not grant untrusted forked pull requests production secrets.

