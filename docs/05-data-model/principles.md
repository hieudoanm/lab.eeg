## 2. Data Modelling Principles

1. Every tenant-owned resource must have an explicit organisation scope.
2. Use UUID primary keys for externally addressable resources.
3. Use foreign keys to enforce relational integrity.
4. Use database constraints for important invariants.
5. Use UTC timestamps for persisted event times.
6. Store scientific metadata explicitly rather than relying on filenames.
7. Keep source recordings immutable after successful registration.
8. Version analysis algorithms and result schemas.
9. Preserve provenance for completed analyses.
10. Avoid storing secrets or unnecessary sensitive information.
11. Prefer explicit SQL and reviewed migrations.
12. Never rely solely on frontend validation for data integrity.

