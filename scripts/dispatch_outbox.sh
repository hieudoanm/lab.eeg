#!/usr/bin/env bash
#
# Recover stale analysis jobs and publish pending outbox events to the broker.
#
# Usage:
#   ./scripts/dispatch_outbox.sh [--skip-recovery] [--batch-size 100] [--max-batches 20]
#
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

uv --directory "${repo_root}/packages/backend" run python -m app.workers.outbox_dispatcher "$@"
