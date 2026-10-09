## 10. `datasets`

Represents a registered recording and its metadata.

Suggested columns:

| Column                  | Type                       | Constraints / purpose                      |
| ----------------------- | -------------------------- | ------------------------------------------ |
| `id`                    | UUID                       | Primary key                                |
| `organisation_id`       | UUID                       | Tenant scope                               |
| `study_id`              | UUID                       | Parent study                               |
| `name`                  | TEXT                       | User-facing dataset name                   |
| `description`           | TEXT, nullable             | Optional description                       |
| `status`                | TEXT                       | Upload and validation lifecycle            |
| `format`                | TEXT                       | Supported recording format                 |
| `storage_key`           | TEXT, nullable             | Private object key                         |
| `object_version`        | TEXT, nullable             | Object version or generation, if supported |
| `checksum_sha256`       | TEXT, nullable             | Content integrity hash                     |
| `file_size_bytes`       | BIGINT, nullable           | Source size                                |
| `sampling_frequency_hz` | DOUBLE PRECISION, nullable | Sampling frequency                         |
| `channel_count`         | INTEGER, nullable          | Number of channels                         |
| `sample_count`          | BIGINT, nullable           | Samples per channel                        |
| `duration_seconds`      | DOUBLE PRECISION, nullable | Recording duration                         |
| `channel_names`         | JSONB, nullable            | Ordered channel labels                     |
| `channel_units`         | JSONB, nullable            | Units or unit metadata                     |
| `metadata`              | JSONB                      | Validated supplementary metadata           |
| `source_description`    | TEXT, nullable             | Provenance description                     |
| `uploaded_by`           | UUID                       | Uploading user                             |
| `created_at`            | TIMESTAMPTZ                | Creation time                              |
| `updated_at`            | TIMESTAMPTZ                | Last update                                |
| `validated_at`          | TIMESTAMPTZ, nullable      | Successful validation time                 |
| `archived_at`           | TIMESTAMPTZ, nullable      | Optional archive time                      |

Suggested states:

- `pending_upload`
- `validating`
- `ready`
- `rejected`
- `failed`
- `archived`

Constraints:

- Dataset name must not be blank.
- File size must be non-negative when present.
- Sampling frequency must be positive when present.
- Channel count must be positive when present.
- Sample count must be non-negative when present.
- Duration must be non-negative when present.
- Checksum must match the documented SHA-256 representation when present.
- `ready` requires valid recording metadata and a registered source object.
- The parent study must belong to the same organisation.
- The uploading user must be a valid user.

### Composite tenant constraints

To reduce the risk of inconsistent cross-organisation relationships, create a unique key on `(organisation_id, id)` and use composite foreign keys where appropriate.

For example, a dataset should reference its parent study through both `organisation_id` and `study_id`, not merely through `study_id`.

This makes it harder to accidentally associate a dataset with a study from another organisation.

### Indexes

- `(organisation_id, study_id, created_at DESC)`
- `(organisation_id, status, updated_at DESC)`
- Unique or supporting index on a stable dataset identifier where required.

Avoid indexing every metadata field by default.

