## 19. Deletion and Retention

Define retention rules before deploying real research data.

Recommended initial rules:

- Archive studies instead of permanently deleting them.
- Preserve completed analysis results and provenance.
- Expire incomplete upload sessions.
- Clean up unreferenced temporary objects after a safe delay.
- Ensure object deletion is coordinated with database state.
- Keep audit retention separate from ordinary resource lifecycle.
- Document how a user's removal affects ownership and attribution records.

If permanent deletion is later implemented, require appropriate authorisation, explicit confirmation, audit logging, and a reliable process for deleting related object-storage data.

