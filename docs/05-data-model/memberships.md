## 8. `memberships`

Associates a user with an organisation and defines their role.

Suggested columns:

| Column            | Type        | Constraints / purpose              |
| ----------------- | ----------- | ---------------------------------- |
| `id`              | UUID        | Primary key                        |
| `organisation_id` | UUID        | Foreign key                        |
| `user_id`         | UUID        | Foreign key                        |
| `role`            | TEXT        | `owner`, `researcher`, or `viewer` |
| `created_at`      | TIMESTAMPTZ | Membership creation                |
| `updated_at`      | TIMESTAMPTZ | Last role update                   |

Constraints:

- Unique `(organisation_id, user_id)`.
- Role must be one of the supported values.
- Both foreign keys must reference existing records.

The last-owner invariant requires transactional application logic or another explicitly enforced mechanism. A simple role check constraint cannot ensure that an organisation always has an owner.

Role changes and membership removals must generate audit events.

