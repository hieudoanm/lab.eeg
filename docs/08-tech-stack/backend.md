## 5. Backend

### 5.1 Python

Use a supported stable Python release compatible with the selected dependency set.

Python is suitable for the API, scientific data processing, and integration with established numerical libraries.

Pin the Python version in project tooling, Docker images, and CI configuration.

### 5.2 FastAPI

Use FastAPI for the REST API.

Responsibilities:

- Authentication integration.
- Request validation.
- Authorisation.
- Organisation-scoped resource access.
- Business rules.
- Dataset metadata management.
- Analysis job submission.
- Job and result retrieval.
- OpenAPI generation.
- Health and readiness endpoints.

Route handlers should remain thin. Put substantial business logic into service modules and persistence logic into repository modules.

### 5.3 Pydantic

Use Pydantic for:

- API request models.
- API response models.
- Environment and application configuration.
- Metadata validation.
- Analysis parameter validation.

Define separate request and response models where their security or lifecycle requirements differ.

Do not expose internal database records directly as public API responses.

### 5.4 Asynchronous execution

Use asynchronous FastAPI route handlers for I/O-bound operations.

Do not run expensive numerical computations directly on the event loop.

Delegate computational work to Celery workers. CPU-intensive work should not block the API process.

### 5.5 Error handling

Use consistent API error responses with:

- A stable error code.
- A human-readable message.
- Optional structured field errors.
- A request identifier where appropriate.

Do not return stack traces, SQL statements, credentials, or internal storage paths.

