## 16. Observability

Every request should have a correlation or request ID.

Log:

- Request ID.
- Route template and HTTP method.
- Response status.
- Duration.
- Relevant resource IDs.
- Safe error codes.

Avoid logging:

- Authentication tokens.
- Signed upload URLs.
- Raw EEG data.
- Unnecessary participant information.
- Full request bodies containing sensitive data.

Background workers should propagate correlation identifiers when available so API requests, outbox events, and analysis executions can be traced together.

The implementation exposes Prometheus metrics at `GET /metrics` (also accepts `HEAD`). This endpoint is unauthenticated and must be restricted to trusted monitoring networks by deployment controls.

