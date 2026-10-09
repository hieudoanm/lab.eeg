## 14. Validation and Pydantic Schemas

Define request and response schemas explicitly.

Suggested schema modules:

```text
app/
  schemas/
    common.py
    auth.py
    organisations.py
    memberships.py
    studies.py
    datasets.py
    uploads.py
    analyses.py
    signals.py
    summary.py
    errors.py
```

Use Pydantic models for:

- Required and optional fields.
- String length limits.
- Numeric bounds.
- Supported status values.
- Nested metadata validation.
- Analysis parameter validation.
- Response serialisation.

Reject unknown request fields for write operations where practical. This reduces accidental acceptance of unsupported client input.

Do not use database models directly as public API response models.

