"""Idempotency-key persistence.

A reservation row is inserted in the same transaction as the operation it
protects. The unique scope ``(organisation_id, user_id, operation, key)`` makes
concurrent duplicate requests safe: one caller inserts, the others observe the
existing row and either replay it (identical request) or reject it (different
request hash).
"""

from __future__ import annotations

from datetime import datetime
from typing import Any
from uuid import UUID

from psycopg import AsyncConnection
from psycopg.types.json import Jsonb

from app.models.analysis import IdempotencyRecord

_CONNECTION = AsyncConnection[dict[str, Any]]

_RECORD_COLUMNS = (
    "id, organisation_id, user_id, operation, key, request_hash, response_status, "
    "response_body, resource_id, created_at, expires_at"
)


def _to_record(row: dict[str, Any]) -> IdempotencyRecord:
    return IdempotencyRecord(
        id=row["id"],
        organisation_id=row["organisation_id"],
        user_id=row["user_id"],
        operation=row["operation"],
        key=row["key"],
        request_hash=row["request_hash"],
        response_status=row["response_status"],
        response_body=row["response_body"],
        resource_id=row["resource_id"],
        created_at=row["created_at"],
        expires_at=row["expires_at"],
    )


async def reserve(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    user_id: UUID,
    operation: str,
    key: str,
    request_hash: str,
    expires_at: datetime,
) -> IdempotencyRecord | None:
    """Insert a reservation, returning ``None`` when the key is already used."""
    cursor = await connection.execute(
        f"""
        INSERT INTO idempotency_keys (
            organisation_id, user_id, operation, key, request_hash, expires_at
        )
        VALUES (%s, %s, %s, %s, %s, %s)
        ON CONFLICT (organisation_id, user_id, operation, key) DO NOTHING
        RETURNING {_RECORD_COLUMNS}
        """,
        (organisation_id, user_id, operation, key, request_hash, expires_at),
    )
    row = await cursor.fetchone()
    return _to_record(row) if row is not None else None


async def get_record(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    user_id: UUID,
    operation: str,
    key: str,
) -> IdempotencyRecord | None:
    """Return an existing reservation for the given scope, if any."""
    cursor = await connection.execute(
        f"""
        SELECT {_RECORD_COLUMNS}
        FROM idempotency_keys
        WHERE organisation_id = %s AND user_id = %s AND operation = %s AND key = %s
        """,
        (organisation_id, user_id, operation, key),
    )
    row = await cursor.fetchone()
    return _to_record(row) if row is not None else None


async def store_response(
    connection: _CONNECTION,
    *,
    record_id: UUID,
    response_status: int,
    response_body: dict[str, Any],
    resource_id: UUID,
) -> None:
    """Attach the safe response representation to a reservation."""
    await connection.execute(
        """
        UPDATE idempotency_keys
        SET response_status = %s, response_body = %s, resource_id = %s
        WHERE id = %s
        """,
        (response_status, Jsonb(response_body), resource_id, record_id),
    )


__all__ = [
    "get_record",
    "reserve",
    "store_response",
]
