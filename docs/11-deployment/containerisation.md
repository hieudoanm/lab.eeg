## 5. Containerisation

Use Docker images for the API and worker. Containerise the frontend if that matches the chosen Next.js deployment model.

### Container requirements

- Use a maintained, supported base image.
- Pin dependency versions.
- Prefer multi-stage builds where useful.
- Run as a non-root user.
- Avoid including development dependencies in production images unnecessarily.
- Do not copy `.env` files or credentials into images.
- Avoid placing scientific recordings in container layers.
- Use a predictable working directory.
- Define health checks where appropriate.
- Handle termination signals so workers can shut down cleanly.

### Example container layout

```text
containers/
├── web.Dockerfile
├── api.Dockerfile
└── worker.Dockerfile
```

The API and worker may share a Python dependency base while retaining separate runtime commands and permissions.

Do not assume that sharing an image means sharing the same execution role.

