"""Audit-event persistence.

Audit records are written inside the same transaction as the mutation they
describe, so an audited change and its audit trail commit or roll back together.
"""

from __future__ import annotations

from typing import Any
from uuid import UUID

from psycopg import AsyncConnection
from psycopg.types.json import Jsonb

_CONNECTION = AsyncConnection[dict[str, Any]]


async def record_audit_event(
    connection: _CONNECTION,
    *,
    action: str,
    resource_type: str,
    organisation_id: UUID | None = None,
    actor_user_id: UUID | None = None,
    resource_id: UUID | None = None,
    request_id: str | None = None,
    metadata: dict[str, Any] | None = None,
) -> None:
    """Append an immutable audit event within the caller's transaction."""
    await connection.execute(
        """
        INSERT INTO audit_events (
            organisation_id, actor_user_id, action, resource_type,
            resource_id, request_id, metadata
        )
        VALUES (%s, %s, %s, %s, %s, %s, %s)
        """,
        (
            organisation_id,
            actor_user_id,
            action,
            resource_type,
            resource_id,
            request_id,
            Jsonb(metadata or {}),
        ),
    )
