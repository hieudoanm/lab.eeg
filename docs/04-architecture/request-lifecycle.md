## 6. Request Lifecycle

For a protected API request:

Browser-originated requests first reach the Next.js tRPC Backend-for-Frontend, which authenticates the caller from the HttpOnly session cookie and resolves the organisation context before forwarding a REST request to the Python API. The steps below describe the Python API that ultimately serves the data.

1. The frontend sends an HTTP request.
2. The API validates the request format.
3. Authentication identifies the caller.
4. The application resolves the relevant organisation context.
5. The server verifies membership and role permissions.
6. The application service executes the requested operation.
7. Repositories perform parameterised SQL operations as required.
8. External services are called only when needed.
9. The API constructs an explicit response model.
10. Structured logs capture the outcome and request identifier.

For state-changing requests, database transactions must protect the relevant invariants.

Authentication, organisation scope, and resource ownership should be checked before returning protected data or starting expensive work.

