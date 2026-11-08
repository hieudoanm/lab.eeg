# Neuro Lab

Neuro Lab is a local-first desktop workspace for EEG, MEG, and MRI research data. It has no server dependency: the React interface runs inside Tauri and calls typed Rust commands directly. All computation and persistence stay on the local filesystem.

## Repository layout

```text
crates/
  neuro-core/       Shared identifiers, provenance, checksums, run state
  neuro-eeg/        EEG readers (NPY, EDF+/EDF, BrainVision), pipelines
                    (filter, PSD, bad channels, re-reference, baseline,
                    epoching/ERP, ICA/FastICA with topographies,
                    SSP), EDF export
  neuro-meg/        MEG readers (FIF, CTF .ds, KIT .sqd/.con, BIDS, OPM),
                    compensation/projectors, events, pipelines
                    (filter, PSD, epoching/evoked, synthetic gradiometers,
                    ICA, SSP, low-order SSS), NPY/CSV/EDF/FIF/BIDS export
  neuro-mri/        NIfTI + DICOM input, slices, projections, statistics,
                    threshold/smooth/mask/segment/resample/z-score ops,
                    bias-field correction, heuristic skull stripping,
                    rigid/affine registration (NCC or mutual information)
  neuro-store/      Local workspace persistence: projects, datasets,
                    dataset versions, experiments, runs, artifacts,
                    upload sessions
packages/desktop/   Tauri + React + TypeScript frontend
reference/          Imported source repositories retained for migration reference
```

## Features

- Local workspace with projects, experiments, datasets, versioned artifacts,
  provenance records, and resumable chunked uploads.
- Background analysis runs with progress reporting and cancellation.
- EEG: NPY/EDF/BrainVision input, EDF+ annotations and `.vmrk` markers as
  events, filtering, PSD, bad-channel detection, re-referencing
  (average/CAR/mastoid/median/robust/Laplacian), baseline correction,
  event-locked epoching with ERP, artifact rejection, FastICA.
- MEG: FIF/CTF/KIT/BIDS/OPM input, CTF compensation grades and FIF
  projectors/compensation applied to data, KIT trigger events and
  `.elp`/`.hsp` digitization sidecars, BIDS multi-run concatenation and
  per-run channel reconciliation, epoching/evoked responses, synthetic
  gradiometers, ICA artifact rejection, SSP, a documented low-order SSS
  approximation for gross interference rejection, and NPY/CSV/EDF/FIF/BIDS
  export (the FIF writer is validated against MNE).
- MRI: NIfTI and DICOM input (native 8/16-bit, multi-frame, RLE, JPEG
  baseline/extended, series directories), multiplanar slices, MIP,
  statistics, histogram, single operations and multi-step pipeline chains,
  bias-field correction, heuristic skull stripping, and rigid/affine
  registration with NCC or mutual-information metrics.
- Dataset search/sort, file-set validation, and artifact export.

## Develop

```bash
pnpm install
pnpm tauri:dev
```

Rust workspace checks:

```bash
cargo fmt --all
cargo test --workspace
cargo check --workspace
```

Frontend checks (from `packages/desktop`):

```bash
pnpm typecheck
pnpm test
pnpm build
pnpm test:e2e
```

## Migration status

The Python/FastAPI implementations in `reference/` are retained as the
authoritative source for workflow parity. All desktop functionality is
implemented in Rust under `crates/`; the server-era modules are not
launched by the application.
