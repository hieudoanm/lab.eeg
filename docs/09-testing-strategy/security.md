## 18. Security Testing

Include checks for:

- Cross-tenant access.
- Broken object-level authorisation.
- SQL injection resistance.
- Unsafe file parsing.
- Oversized uploads.
- Malformed JSON and metadata.
- Excessive resource consumption.
- Secret leakage.
- Signed URL leakage.
- Unauthorised role changes.
- Unsafe error responses.
- Dependency vulnerabilities.
- Container image vulnerabilities.
- Public exposure of private object storage.
- Insecure default configuration.

All SQL values must be parameterised. Dynamic SQL identifiers, when unavoidable, must be selected from trusted allowlists rather than interpolated from user input.

