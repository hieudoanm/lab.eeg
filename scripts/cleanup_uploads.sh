#!/usr/bin/env bash
#
# Expire abandoned upload sessions and delete their stored recording objects.
#
# Usage:
#   ./scripts/cleanup_uploads.sh [--batch-size 100] [--max-batches 50]
#
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

uv --directory "${repo_root}/packages/backend" run python -m app.workers.cleanup "$@"
