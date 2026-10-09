"""Outbox persistence: durable dispatch records for background work.

Outbox events are inserted in the same transaction as the state change they
announce, so a committed job always has a dispatchable event and a rolled-back
job never does. A dispatcher claims pending rows with ``FOR UPDATE SKIP LOCKED``,
publishes them, and records the outcome.
"""

from __future__ import annotations

from datetime import datetime
from typing import Any
from uuid import UUID

from psycopg import AsyncConnection
from psycopg.types.json import Jsonb

from app.models.analysis import OutboxEvent

_CONNECTION = AsyncConnection[dict[str, Any]]

_EVENT_COLUMNS = (
    "id, organisation_id, event_type, aggregate_type, aggregate_id, payload, "
    "created_at, published_at, attempt_count, next_attempt_at, last_error_code"
)


def _to_event(row: dict[str, Any]) -> OutboxEvent:
    return OutboxEvent(
        id=row["id"],
        organisation_id=row["organisation_id"],
        event_type=row["event_type"],
        aggregate_type=row["aggregate_type"],
        aggregate_id=row["aggregate_id"],
        payload=row["payload"] or {},
        created_at=row["created_at"],
        published_at=row["published_at"],
        attempt_count=row["attempt_count"],
        next_attempt_at=row["next_attempt_at"],
        last_error_code=row["last_error_code"],
    )


async def create_event(
    connection: _CONNECTION,
    *,
    organisation_id: UUID | None,
    event_type: str,
    aggregate_type: str,
    aggregate_id: UUID,
    payload: dict[str, Any],
) -> OutboxEvent:
    """Append a pending dispatch event within the caller's transaction."""
    cursor = await connection.execute(
        f"""
        INSERT INTO outbox_events (
            organisation_id, event_type, aggregate_type, aggregate_id, payload
        )
        VALUES (%s, %s, %s, %s, %s)
        RETURNING {_EVENT_COLUMNS}
        """,
        (organisation_id, event_type, aggregate_type, aggregate_id, Jsonb(payload)),
    )
    row = await cursor.fetchone()
    assert row is not None
    return _to_event(row)


async def claim_pending_events(
    connection: _CONNECTION,
    *,
    now: datetime,
    limit: int,
) -> list[OutboxEvent]:
    """Lock and return a batch of undispatched, due events.

    Rows are locked with ``FOR UPDATE SKIP LOCKED`` so concurrent dispatchers
    never publish the same event. The lock is held for the duration of the
    caller's transaction, which spans the publish call, guaranteeing that a
    successfully published event is marked before the lock is released.
    """
    cursor = await connection.execute(
        f"""
        SELECT {_EVENT_COLUMNS}
        FROM outbox_events
        WHERE published_at IS NULL
          AND (next_attempt_at IS NULL OR next_attempt_at <= %s)
        ORDER BY created_at, id
        LIMIT %s
        FOR UPDATE SKIP LOCKED
        """,
        (now, limit),
    )
    rows = await cursor.fetchall()
    return [_to_event(row) for row in rows]


async def mark_published(
    connection: _CONNECTION,
    *,
    event_id: UUID,
    published_at: datetime,
) -> None:
    """Record successful publication."""
    await connection.execute(
        """
        UPDATE outbox_events
        SET published_at = %s, next_attempt_at = NULL, last_error_code = NULL
        WHERE id = %s
        """,
        (published_at, event_id),
    )


async def mark_dispatch_failed(
    connection: _CONNECTION,
    *,
    event_id: UUID,
    next_attempt_at: datetime,
    error_code: str,
) -> None:
    """Record a failed dispatch attempt and schedule a bounded retry."""
    await connection.execute(
        """
        UPDATE outbox_events
        SET attempt_count = attempt_count + 1,
            next_attempt_at = %s,
            last_error_code = %s
        WHERE id = %s
        """,
        (next_attempt_at, error_code, event_id),
    )


__all__ = [
    "claim_pending_events",
    "create_event",
    "mark_dispatch_failed",
    "mark_published",
]
