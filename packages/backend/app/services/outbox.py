"""Outbox dispatch: publish durable events to the Celery broker.

A dispatcher claims pending events with ``FOR UPDATE SKIP LOCKED``, publishes
each to the broker, and records the outcome — all inside one short transaction
per batch. The row lock is held across the publish call so two dispatchers can
never publish the same event. A failed publish is retried with bounded
exponential backoff; an exhausted event is parked with a visible error code.
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from typing import Any, Protocol

from app.db.pool import Database
from app.models.analysis import ANALYSIS_EVENT_TASK_NAMES
from app.repositories import outbox as outbox_repository

#: How far in the future an exhausted event is parked so it stops retrying.
_PARKED_HORIZON = timedelta(days=365 * 100)

#: Error code recorded when a publish repeatedly fails and retries are exhausted.
_DISPATCH_EXHAUSTED_CODE = "outbox_dispatch_exhausted"

#: Error code recorded when an event's type has no registered handler.
_UNKNOWN_EVENT_CODE = "outbox_unknown_event"


class EventPublisher(Protocol):
    """The minimal broker surface the dispatcher depends on."""

    def publish(self, *, task_name: str, payload: dict[str, Any]) -> None:
        """Enqueue a task; raise on failure."""


@dataclass(frozen=True, slots=True)
class DispatchReport:
    """Summary of an outbox dispatch run."""

    claimed: int
    published: int
    failed: int
    exhausted: int
    batches: int


def _now() -> datetime:
    return datetime.now(UTC)


async def dispatch_pending_events(
    database: Database,
    publisher: EventPublisher,
    *,
    now: datetime | None = None,
    batch_size: int = 100,
    max_batches: int = 20,
    max_attempts: int = 10,
    retry_delay_seconds: float = 30.0,
    task_names: Mapping[str, str] | None = None,
) -> DispatchReport:
    """Publish pending outbox events, returning a summary of the run."""
    if batch_size < 1:
        raise ValueError("batch_size must be at least 1")
    if max_batches < 1:
        raise ValueError("max_batches must be at least 1")

    deadline = now or _now()
    registry = dict(task_names if task_names is not None else ANALYSIS_EVENT_TASK_NAMES)

    claimed = published = failed = exhausted = batches = 0
    while batches < max_batches:
        async with database.transaction() as connection:
            events = await outbox_repository.claim_pending_events(
                connection,
                now=deadline,
                limit=batch_size,
            )
            if not events:
                break
            claimed += len(events)
            for event in events:
                task_name = registry.get(event.event_type)
                if task_name is None:
                    await outbox_repository.mark_dispatch_failed(
                        connection,
                        event_id=event.id,
                        next_attempt_at=deadline + _PARKED_HORIZON,
                        error_code=_UNKNOWN_EVENT_CODE,
                    )
                    exhausted += 1
                    continue
                try:
                    publisher.publish(task_name=task_name, payload=event.payload)
                except Exception:
                    if event.attempt_count + 1 >= max_attempts:
                        await outbox_repository.mark_dispatch_failed(
                            connection,
                            event_id=event.id,
                            next_attempt_at=deadline + _PARKED_HORIZON,
                            error_code=_DISPATCH_EXHAUSTED_CODE,
                        )
                        exhausted += 1
                    else:
                        delay = retry_delay_seconds * (2**event.attempt_count)
                        await outbox_repository.mark_dispatch_failed(
                            connection,
                            event_id=event.id,
                            next_attempt_at=deadline + timedelta(seconds=delay),
                            error_code="outbox_dispatch_failed",
                        )
                        failed += 1
                else:
                    await outbox_repository.mark_published(
                        connection,
                        event_id=event.id,
                        published_at=deadline,
                    )
                    published += 1
        batches += 1
        if len(events) < batch_size:
            break

    return DispatchReport(
        claimed=claimed,
        published=published,
        failed=failed,
        exhausted=exhausted,
        batches=batches,
    )


__all__ = [
    "DispatchReport",
    "EventPublisher",
    "dispatch_pending_events",
]
