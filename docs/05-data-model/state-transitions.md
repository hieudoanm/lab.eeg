## 18. State Transitions

### Dataset lifecycle

```text
pending_upload
    ├── validating
    │     ├── ready
    │     ├── rejected
    │     └── failed
    ├── failed
    └── expired or cleaned up according to policy

ready
    └── archived
```

Exact transitions depend on the implemented upload strategy. Define allowed transitions in application code and enforce critical invariants at the database boundary.

### Analysis job lifecycle

```text
queued
   ├── running
   │     ├── succeeded
   │     ├── failed
   │     └── queued again when a retry is explicitly permitted
   └── cancelled

running
   └── recovered according to lease and retry policy
```

A cancelled or successful job must not be silently reopened by a duplicate worker message.

If retrying creates a new execution record, preserve the original execution history.

