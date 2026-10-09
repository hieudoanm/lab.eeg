"""Celery tasks for asynchronous analysis execution.

``run_analysis`` is intentionally thin: it builds the runtime dependencies and
delegates to ``execute_analysis_job``, which owns the claim/execute/commit
protocol against PostgreSQL. Because retries are driven by the durable job's
attempt count (a retry requeues the job and emits a fresh outbox event), the task
does not rely on Celery's in-memory retry state.
"""

from __future__ import annotations

import asyncio
import logging
from uuid import UUID

from app.core.config import get_settings
from app.db.pool import Database
from app.models.analysis import ANALYSIS_TASK_NAME
from app.services.analysis_execution import ExecutionOutcome, execute_analysis_job
from app.storage import build_storage
from app.workers.celery_app import celery_app

logger = logging.getLogger("eeg-lab.worker")


async def run_analysis_job(job_id: UUID) -> ExecutionOutcome:
    """Execute one analysis job with a short-lived database pool."""
    settings = get_settings()
    database = Database(settings)
    await database.connect()
    try:
        storage = build_storage(settings)
        return await execute_analysis_job(database, storage, settings, job_id=job_id)
    finally:
        await database.close()


@celery_app.task(name=ANALYSIS_TASK_NAME)
def run_analysis(job_id: str) -> None:
    """Celery entry point: execute the analysis job identified by ``job_id``."""
    outcome = asyncio.run(run_analysis_job(UUID(job_id)))
    logger.info(
        "analysis task finished",
        extra={"job_id": job_id, "outcome": outcome.status, "result_id": str(outcome.result_id)},
    )


__all__ = ["run_analysis", "run_analysis_job"]
