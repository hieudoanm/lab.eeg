## 16. Rollback Strategy

Application rollback and data rollback are separate operations.

### Application rollback

Keep previous immutable application images available. If a deployment fails, restore the previous compatible version and run smoke tests.

### Database rollback

Do not automatically reverse a migration without checking its effects.

Prefer backward-compatible migrations that allow the application to be rolled back without losing data.

For destructive changes, define a specific recovery plan before deployment.

### Worker compatibility

During a rollout, old and new workers may temporarily coexist.

Ensure that:

- Job payloads remain compatible during the transition.
- Algorithm versions are explicit.
- Result schemas are versioned.
- Workers cannot overwrite results from incompatible executions.
- Queued jobs remain processable after deployment.

