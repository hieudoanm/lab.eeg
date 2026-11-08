# Agent guide

## Product boundary

Neuro Lab is local-only. Do not add an API server, PostgreSQL, Redis, Celery, cloud storage, authentication service, or remote telemetry. The frontend must communicate with Rust through Tauri commands.

## Rust conventions

- Keep modality-independent types and checksums in `crates/neuro-core`.
- Keep EEG, MEG, and MRI parsing and scientific logic in their own crates.
- Expose narrow, serializable operations from `packages/desktop/src-tauri`.
- Preserve deterministic behavior, finite-sample validation, bounded reads, provenance, and checksums.
- Never claim clinical findings.

## Frontend conventions

- Strict TypeScript under `packages/desktop/src`.
- Follow the 20-rule design guidance in `hieudoanm/notes/ai-unslop-design.md`: concrete labels, purposeful spacing, consistent controls, accessible states, tabular numeric data, and visible loading/error feedback.
- Every asynchronous operation must expose loading and error states.

## Verification

Run `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo check --workspace`, `pnpm typecheck`, `pnpm build`, and `pnpm tauri:build` when dependencies are available.
