## 16. Contract and Type Consistency Tests

The API is defined by FastAPI and Pydantic.

Where practical:

- Generate an OpenAPI specification in CI.
- Generate TypeScript client types from OpenAPI.
- Check generated files for unexpected changes.
- Validate representative API responses against the contract.
- Ensure frontend and backend agree on status values and error codes.

Avoid maintaining duplicated type definitions when automated generation is reliable and easier to maintain.

