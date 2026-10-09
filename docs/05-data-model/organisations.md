## 7. `organisations`

Represents an organisation or research workspace.

Suggested columns:

| Column       | Type        | Constraints / purpose              |
| ------------ | ----------- | ---------------------------------- |
| `id`         | UUID        | Primary key                        |
| `name`       | TEXT        | Required, non-empty                |
| `slug`       | TEXT        | Optional human-readable identifier |
| `status`     | TEXT        | Active or archived                 |
| `created_at` | TIMESTAMPTZ | Creation time                      |
| `updated_at` | TIMESTAMPTZ | Last update time                   |

Constraints:

- Name must not be blank.
- Status must use a supported value.
- If a slug is used, define whether uniqueness is global or organisation-specific. A globally unique slug is the simpler initial choice.

Do not rely on the slug as a security boundary.

