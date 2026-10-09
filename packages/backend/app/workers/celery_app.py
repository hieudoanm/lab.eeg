"""Celery application for the analysis worker.

The broker (Redis) only transports dispatch messages. PostgreSQL remains the
authoritative source of job state and results, so the worker never trusts the
broker for correctness: Redis is configured without a result backend.
"""

from __future__ import annotations

from celery import Celery

from app.core.config import Settings, get_settings


def create_celery_app(settings: Settings | None = None) -> Celery:
    """Build a Celery application from the process configuration."""
    resolved = settings or get_settings()
    app = Celery(
        "eeg-lab",
        broker=resolved.resolved_celery_broker_url,
        include=["app.workers.analysis_tasks"],
    )
    app.conf.update(
        task_time_limit=resolved.celery_task_time_limit_seconds,
        task_soft_time_limit=resolved.celery_task_soft_time_limit_seconds,
        task_acks_late=True,
        task_reject_on_worker_lost=True,
        worker_prefetch_multiplier=1,
        task_serializer="json",
        accept_content=["json"],
        result_backend=None,
        broker_connection_retry_on_startup=True,
        timezone="UTC",
        enable_utc=True,
    )
    return app


celery_app = create_celery_app()


__all__ = ["celery_app", "create_celery_app"]
