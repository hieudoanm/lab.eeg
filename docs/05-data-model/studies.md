## 9. `studies`

Represents a research study.

Suggested columns:

| Column            | Type                  | Constraints / purpose      |
| ----------------- | --------------------- | -------------------------- |
| `id`              | UUID                  | Primary key                |
| `organisation_id` | UUID                  | Owning organisation        |
| `name`            | TEXT                  | Required, non-empty        |
| `description`     | TEXT, nullable        | Optional description       |
| `status`          | TEXT                  | `active` or `archived`     |
| `created_by`      | UUID                  | User who created the study |
| `created_at`      | TIMESTAMPTZ           | Creation time              |
| `updated_at`      | TIMESTAMPTZ           | Last update                |
| `archived_at`     | TIMESTAMPTZ, nullable | Archive timestamp          |

Constraints:

- Name must not be blank.
- Status must be supported.
- Organisation foreign key is required.
- Creator must reference a valid user.

A study name does not need to be globally unique. Multiple organisations may use the same name, and one organisation may have several similarly named studies.

### Indexes

- `(organisation_id, created_at DESC)`
- `(organisation_id, status, updated_at DESC)`

Add search indexes only when required by the implemented search behaviour.

