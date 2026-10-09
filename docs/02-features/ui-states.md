## 13. User Interface States

**Priority:** P0

Every primary screen must implement the states relevant to its workflow.

### Loading

Show progress or a skeleton for asynchronous requests. Avoid misleading empty screens while data is still loading.

### Empty

Explain why there is no content and offer an appropriate next action.

### Success

Confirm the completed operation and show the resulting resource or next step.

### Validation error

Identify the invalid field or file property and explain how to fix it.

### Permission error

Explain that the requested action is unavailable without exposing another tenant's protected resources.

### Network error

Allow safe retry where appropriate and preserve user-entered form data when possible.

### Unexpected server error

Show a safe message and request identifier where available. Do not expose internal stack traces.

