## 8. Configuration and Secrets

### Configuration

Keep non-secret environment-specific settings in explicit configuration.

Examples:

- Environment name.
- Log level.
- API URL.
- Resource limits.
- Supported file formats.
- Analysis concurrency.
- Feature flags.

### Secrets

Use Secret Manager or an equivalent managed secret facility for:

- Database credentials where credentials are required.
- Broker authentication.
- Authentication-provider secrets.
- Third-party API credentials.

Use workload identity or service identities wherever possible to avoid long-lived service-account keys.

### Rules

- Never commit secrets.
- Never log secret values.
- Never expose server secrets through `NEXT_PUBLIC_*` variables.
- Rotate credentials according to a documented policy.
- Use separate credentials for local, CI, and production.
- Grant each service access only to the secrets it requires.

