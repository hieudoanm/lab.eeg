## 2. Delivery Principles

### Build a vertical slice first

The first objective is not to build every screen or establish the entire cloud infrastructure.

Build one complete workflow:

1. Open the application.
2. Authenticate or use an explicitly configured local development identity.
3. Create a study.
4. Register a synthetic EEG dataset.
5. Upload and validate the recording.
6. Submit an analysis.
7. Process it in a background worker.
8. Retrieve and visualise the result.

This workflow proves that the major components work together.

### Prefer working software over premature abstraction

- Implement only the features required by the current milestone.
- Avoid building a generic workflow engine.
- Avoid introducing a complex permission framework before the initial roles are understood.
- Avoid building a plugin system for a single analysis algorithm.
- Add abstractions when repeated behaviour or a clear architectural boundary justifies them.

### Keep scientific claims modest

The initial application performs descriptive EEG analysis. It does not diagnose conditions, interpret clinical outcomes, or claim that its outputs are validated for medical use.

### Protect the development feedback loop

- Run linting and type checking frequently.
- Add tests alongside features.
- Use small pull requests or commits.
- Keep the local development environment reproducible.
- Ensure a broken master branch is treated as a delivery problem.

