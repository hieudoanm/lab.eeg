# Desktop app

Prebuilt bundles are attached to the
[latest GitHub release](https://github.com/hieudoanm/neuro.lab/releases/latest).

## Install

```bash
scripts/install.sh        # download a release bundle, or build from source
```

## From source

```bash
pnpm install
pnpm tauri:dev            # development
pnpm tauri:build          # production bundles under target/release/bundle
```

## Prerequisites

- Node.js (see `.nvmrc`) and pnpm
- Rust stable toolchain
