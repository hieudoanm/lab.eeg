## 15. Rate Limits and Resource Limits

Apply limits to resource-intensive operations.

Initial policies should cover:

- Upload size.
- Upload-session creation frequency.
- Analysis submissions per user or organisation.
- Maximum concurrent jobs.
- Maximum array dimensions and sample counts.
- Request body size.
- Pagination limits.
- Authentication failures.

Return `429 Too Many Requests` when a rate limit is exceeded, including `Retry-After` where appropriate.

Specific limits must be configurable and documented. Do not rely on application rate limiting as the only protection against excessive resource use. Rate limiting is not implemented in the current API; the checked-in OpenAPI artifact therefore does not advertise `429` responses until a limiter is wired into the request lifecycle.

