# Contributing to neuro.lab

Thanks for contributing to Neuro Lab, a local-first desktop workspace for EEG,
MEG, and MRI research data.

## Product boundary

Neuro Lab is local-only. Do not add an API server, PostgreSQL, Redis, Celery,
cloud storage, authentication service, or remote telemetry. The frontend must
communicate with Rust through Tauri commands.

## Repository layout

- `crates/` — Rust workspace; modality-independent types in `neuro-core`,
  EEG/MEG/MRI logic in their own crates, narrow serializable operations in
  `packages/desktop/src-tauri`.
- `packages/desktop` — Tauri + React + TypeScript frontend.
- `packages/landing` — static landing page.
- `reference/` — imported source repositories retained for migration
  reference (not launched by the app).

## Setup

```bash
git clone https://github.com/hieudoanm/neuro.lab.git
cd neuro.lab
pnpm install
pnpm tauri:dev      # development
pnpm tauri:build    # production bundles
```

Requires Node.js (see `.nvmrc`), pnpm, and a stable Rust toolchain.

## Checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd packages/desktop
pnpm typecheck && pnpm lint && pnpm test && pnpm build && pnpm test:e2e
```

## Conventions

- Preserve deterministic behavior, finite-sample validation, bounded reads,
  provenance, and checksums.
- Strict TypeScript under `packages/desktop/src`; every async operation must
  expose loading and error states.
- Never claim clinical findings.

See [AGENTS.md](AGENTS.md) for the full guide.

## License

By contributing, you agree that your contributions will be licensed under the
repository license in [`LICENSE`](LICENSE).
