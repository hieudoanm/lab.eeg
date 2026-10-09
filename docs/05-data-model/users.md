## 6. `users`

Represents an application user.

Suggested columns:

| Column              | Type                  | Constraints / purpose            |
| ------------------- | --------------------- | -------------------------------- |
| `id`                | UUID                  | Primary key                      |
| `identity_provider` | TEXT                  | External identity provider       |
| `external_subject`  | TEXT                  | Stable identity-provider subject |
| `email`             | TEXT, nullable        | Contact/display information      |
| `display_name`      | TEXT, nullable        | User-facing name                 |
| `created_at`        | TIMESTAMPTZ           | Creation time                    |
| `updated_at`        | TIMESTAMPTZ           | Last update time                 |
| `disabled_at`       | TIMESTAMPTZ, nullable | Optional account disablement     |

Constraints:

- Unique identity on `(identity_provider, external_subject)`.
- Do not use email as the sole permanent identity key.
- Normalise email consistently if it is used for lookup.
- Do not store passwords when authentication is delegated to an external provider.

A local-development identity mechanism must be isolated from production authentication.

