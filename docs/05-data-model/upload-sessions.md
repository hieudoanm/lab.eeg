## 11. `upload_sessions`

Tracks upload attempts and finalisation.

Suggested columns:

| Column                     | Type                  | Constraints / purpose                   |
| -------------------------- | --------------------- | --------------------------------------- |
| `id`                       | UUID                  | Primary key                             |
| `organisation_id`          | UUID                  | Tenant scope                            |
| `dataset_id`               | UUID                  | Associated dataset                      |
| `created_by`               | UUID                  | Initiating user                         |
| `status`                   | TEXT                  | Upload lifecycle                        |
| `object_key`               | TEXT, nullable        | Server-generated storage key            |
| `manifest`                 | JSONB                 | Channel/sampling manifest, default `{}` |
| `expected_size_bytes`      | BIGINT, nullable      | Expected upload size                    |
| `expected_checksum_sha256` | TEXT, nullable        | Expected checksum                       |
| `expires_at`               | TIMESTAMPTZ           | Expiry                                  |
| `finalised_at`             | TIMESTAMPTZ, nullable | Finalisation time                       |
| `created_at`               | TIMESTAMPTZ           | Creation time                           |
| `updated_at`               | TIMESTAMPTZ           | Last update                             |

Suggested states:

- `created`
- `uploading`
- `uploaded`
- `finalising`
- `completed`
- `failed`
- `expired`

Constraints and rules:

- Upload sessions are scoped to an organisation.
- `manifest` stores the validated channel/sampling metadata supplied at session creation; it is copied to the dataset only after successful validation.
- Expired sessions cannot be finalised as new uploads.
- Finalisation must be idempotent.
- Temporary upload authorisations must expire.
- Abandoned sessions must be discoverable for cleanup.
- Never store a signed URL as a permanent storage reference.

When a session fails validation, the session transitions to `failed` and the associated dataset to `rejected`; only a successful finalisation writes recording metadata and `validated_at` onto the dataset.

