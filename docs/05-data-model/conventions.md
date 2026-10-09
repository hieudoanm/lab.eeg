## 5. Common Column Conventions

### Identifiers

Use PostgreSQL UUID columns for primary keys.

Generate identifiers in the application or through PostgreSQL's supported UUID generation functions. Use one consistent approach.

### Timestamps

Use `TIMESTAMPTZ` for timestamps.

Common fields:

- `created_at`
- `updated_at`
- `completed_at`
- `archived_at`

Use UTC for application-level timestamp handling and return ISO 8601 timestamps through the API.

### Statuses

Represent lifecycle states using constrained text values or PostgreSQL enums.

For an evolving MVP, `TEXT` plus `CHECK` constraints can be easier to migrate than frequently changing enum types. Apply the same approach consistently.

### JSONB

Use `JSONB` for structured, validated metadata when its shape genuinely varies.

Do not use unrestricted JSONB as a substitute for relational modelling of core ownership, identity, status, or frequently queried relationships.

### Deletion

Prefer archival or explicit lifecycle transitions for scientific resources.

Avoid cascading deletion from an organisation or study into irreplaceable scientific data unless permanent deletion is an explicit, authorised, and auditable workflow.

