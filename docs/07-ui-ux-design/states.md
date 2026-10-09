## 18. Loading, Empty, and Error States

Every data-driven screen must distinguish between the following states.

### Loading

Use a progress indicator or skeleton. Avoid flashing an empty state before data arrives.

### Empty

Explain why the collection is empty and provide a next step if the user has permission to take one.

### No results

Explain that the current filter or search found no matching resources.

### Validation error

Identify what needs correcting and preserve relevant form input.

### Permission error

Explain that the resource or operation is unavailable without exposing protected information.

### Network failure

Provide a safe retry mechanism where possible. Do not repeat non-idempotent mutations automatically unless their idempotency is guaranteed.

### Unexpected server error

Display a friendly message and a request identifier when available. Do not expose stack traces, SQL errors, credentials, or internal service details.

