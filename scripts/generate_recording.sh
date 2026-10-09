#!/usr/bin/env bash
#
# Generate a deterministic synthetic EEG recording (.npy + JSON manifest).
#
# Usage:
#   ./scripts/generate_recording.sh --channels 8 --samples 2000 --pattern mixed
#
# Run `./scripts/generate_recording.sh --help` for all options.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

uv --directory "${repo_root}/packages/backend" run python -m app.scientific.synthetic "$@"
