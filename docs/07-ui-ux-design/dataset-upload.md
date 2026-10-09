## 10. Dataset Upload Page

### Purpose

Make uploading and validating a recording understandable and recoverable.

### Layout

Use a focused form rather than a large dashboard.

Suggested sections:

1. Dataset details.
2. File selection.
3. Recording metadata.
4. Upload and validation progress.
5. Final outcome.

### File selection

The file input should clearly display:

- Selected filename.
- File size.
- Supported format.
- Configured size limit.
- Removal or replacement action before submission.

Do not rely exclusively on drag-and-drop. A standard accessible file picker must remain available.

### Validation feedback

Show progress through clearly labelled stages, such as:

```text
Upload → Validation → Ready
```

If validation fails, display:

- A concise explanation.
- Relevant field or file information.
- A corrective action.
- A retry option only when retrying is meaningful.

Do not show a success state until the backend confirms the dataset is ready.

