"""Outbox dispatcher and stale-job recovery command.

Runs the two background maintenance steps that keep analyses moving:

1. Recover jobs whose worker lease expired (requeue or fail).
2. Publish pending outbox events to the Celery broker.

Run with::

    uv --directory packages/backend run python -m app.workers.outbox_dispatcher

or via ``./scripts/dispatch_outbox.sh`` / ``make dispatch-outbox``. Scheduling
this on an interval (Celery beat or a platform cron) belongs to the reliability
phase; this command is the on-demand unit of work.
"""

from __future__ import annotations

import argparse
import asyncio
from typing import Any

from celery import Celery

from app.core.config import Settings, get_settings
from app.db.pool import Database
from app.services.analysis_execution import RecoveryReport, recover_stale_jobs
from app.services.outbox import DispatchReport, dispatch_pending_events
from app.workers.celery_app import celery_app


class CeleryEventPublisher:
    """Publish outbox events as Celery tasks on the shared broker."""

    def __init__(self, app: Celery) -> None:
        self._app = app

    def publish(self, *, task_name: str, payload: dict[str, Any]) -> None:
        job_id = payload.get("job_id")
        if not isinstance(job_id, str) or not job_id:
            raise ValueError("outbox payload is missing a job_id")
        self._app.send_task(task_name, args=[job_id])


async def run_dispatcher(
    settings: Settings,
    *,
    recover: bool,
    batch_size: int,
    max_batches: int,
) -> tuple[RecoveryReport | None, DispatchReport]:
    """Recover stale jobs (optionally) and publish pending outbox events."""
    database = Database(settings)
    await database.connect()
    try:
        recovery = (
            await recover_stale_jobs(
                database,
                settings,
                batch_size=batch_size,
                max_batches=max_batches,
            )
            if recover
            else None
        )
        report = await dispatch_pending_events(
            database,
            CeleryEventPublisher(celery_app),
            batch_size=batch_size,
            max_batches=max_batches,
            max_attempts=settings.outbox_dispatch_max_attempts,
        )
        return recovery, report
    finally:
        await database.close()


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Recover stale analysis jobs and dispatch pending outbox events.",
    )
    parser.add_argument(
        "--skip-recovery",
        action="store_true",
        help="do not run stale-job recovery before dispatching",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=100,
        help="events or jobs claimed per transaction (default: 100)",
    )
    parser.add_argument(
        "--max-batches",
        type=int,
        default=20,
        help="upper bound on batches processed in one run (default: 20)",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    """Command-line entry point for outbox dispatch and recovery."""
    args = _build_parser().parse_args(argv)
    recovery, report = asyncio.run(
        run_dispatcher(
            get_settings(),
            recover=not args.skip_recovery,
            batch_size=args.batch_size,
            max_batches=args.max_batches,
        )
    )
    if recovery is not None:
        print(
            f"Recovery complete: requeued {recovery.requeued} job(s), "
            f"failed {recovery.failed} job(s) in {recovery.batches} batch(es)."
        )
    print(
        f"Dispatch complete: claimed {report.claimed} event(s), "
        f"published {report.published}, retryable failures {report.failed}, "
        f"exhausted {report.exhausted} in {report.batches} batch(es)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())


__all__ = ["CeleryEventPublisher", "main", "run_dispatcher"]
