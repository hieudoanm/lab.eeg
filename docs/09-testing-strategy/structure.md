## 5. Test Directory Structure

A possible repository layout:

```text
eeg-lab/
├── packages/
│   ├── frontend/
│   │   └── src/
│   │       ├── components/
│   │       └── ...
│   └── backend/
│       ├── app/
│       └── tests/
│           ├── unit/
│           ├── integration/
│           ├── api/
│           ├── security/
│           └── fixtures/
├── workers/
│   └── analysis/
│       ├── app/
│       └── tests/
│           ├── unit/
│           ├── integration/
│           └── fixtures/
├── tests/
│   └── e2e/
└── infrastructure/
    └── terraform/
```

Adapt this structure to the actual repository. Avoid duplicating shared fixtures unnecessarily.

