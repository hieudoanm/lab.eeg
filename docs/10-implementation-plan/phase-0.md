## 4. Phase 0 — Project Foundation

### Objectives

Establish a reproducible development environment and repository structure.

### Tasks

- [x] Create the repository and initial README.
- [x] Add `AGENTS.md` with coding and repository conventions.
- [x] Add the 12 core project documentation files.
- [x] Define frontend and backend directory structures.
- [x] Configure Python dependency management.
- [x] Configure pnpm and Node.js versions.
- [x] Add formatting and linting tools.
- [x] Configure Husky and lint-staged for staged-source checks before commit.
- [x] Add `.editorconfig`.
- [x] Add environment variable examples.
- [x] Add a root-level Makefile or equivalent task runner.
- [x] Configure GitHub Actions for initial checks.
- [x] Add a `.gitignore` covering secrets, local data, build outputs, and generated scientific files.

### Suggested repository structure

```text
eeg-lab/
├── packages/
│   ├── frontend/
│   ├── backend/
│   └── shared-contracts/
├── workers/
│   └── analysis/
├── infrastructure/
│   └── terraform/
├── docs/
├── scripts/
├── tests/
├── .github/
│   └── workflows/
├── AGENTS.md
├── Makefile
├── README.md
└── .gitignore
```

The shared-contracts package is optional. Introduce it only if sharing generated API types provides a clear benefit.

### Acceptance criteria

- A new developer can follow the setup guide.
- The frontend and backend start locally.
- Formatting and linting commands run.
- Husky installs its pre-commit hook after dependency installation and checks staged source files.
- CI executes the initial checks.
- No credentials or real participant recordings are committed.
- Project documentation reflects the actual chosen tools.

