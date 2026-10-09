## 17. API Documentation

FastAPI should generate an OpenAPI specification from the implemented routes and Pydantic schemas.
The checked-in artifact is based on that generated output and also documents authentication, the common error handlers, and mounted endpoints such as Prometheus metrics that are not fully represented by the default generator.

Requirements:

- Provide summaries and descriptions for endpoints.
- Document authentication.
- Document request and response schemas.
- Include error responses.
- Document supported enum values and lifecycle states.
- Keep examples consistent with the actual implementation.
- Make interactive API documentation available in development.
- Restrict or disable public interactive documentation in production according to deployment policy.
- Keep the checked-in implementation-derived specification at `docs/api/api-spec.json` in sync with the running FastAPI application.

OpenAPI output should be checked in CI for unintended contract changes where practical.

