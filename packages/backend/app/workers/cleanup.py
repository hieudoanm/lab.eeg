"""Abandoned-upload cleanup command.

Expires upload sessions that exceeded their TTL and deletes any recording bytes
they left in object storage. This is the on-demand counterpart to the lazy
per-dataset expiry performed during upload-session creation; scheduling it
periodically belongs to the reliability phase.

Run with::

    uv --directory packages/backend run python -m app.workers.cleanup

or via ``./scripts/cleanup_uploads.sh`` / ``make cleanup-uploads``.
"""

from __future__ import annotations

import argparse
import asyncio

from app.core.config import Settings, get_settings
from app.db.pool import Database
from app.services.uploads import CleanupReport, cleanup_abandoned_uploads
from app.storage import build_storage


async def run_cleanup(settings: Settings, *, batch_size: int, max_batches: int) -> CleanupReport:
    """Connect, clean up abandoned uploads, and release resources."""
    database = Database(settings)
    await database.connect()
    try:
        storage = build_storage(settings)
        return await cleanup_abandoned_uploads(
            database,
            storage,
            batch_size=batch_size,
            max_batches=max_batches,
        )
    finally:
        await database.close()


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Expire abandoned upload sessions and delete their stored objects.",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=100,
        help="sessions claimed per transaction (default: 100)",
    )
    parser.add_argument(
        "--max-batches",
        type=int,
        default=50,
        help="upper bound on batches processed in one run (default: 50)",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    """Command-line entry point for abandoned-upload cleanup."""
    args = _build_parser().parse_args(argv)
    report = asyncio.run(
        run_cleanup(get_settings(), batch_size=args.batch_size, max_batches=args.max_batches)
    )
    print(
        f"Cleanup complete: expired {report.expired_sessions} session(s), "
        f"deleted {report.objects_deleted} object(s), "
        f"failed {report.objects_failed} object(s) in {report.batches} batch(es)."
    )
    return 1 if report.objects_failed else 0


if __name__ == "__main__":
    raise SystemExit(main())


__all__ = ["main", "run_cleanup"]
