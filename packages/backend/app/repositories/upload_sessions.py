"""Upload session persistence: explicit SQL and row mapping."""

from __future__ import annotations

from datetime import datetime
from typing import Any
from uuid import UUID

from psycopg import AsyncConnection
from psycopg.types.json import Jsonb

from app.models.upload_session import UploadSession, UploadSessionStatus

_CONNECTION = AsyncConnection[dict[str, Any]]

_SESSION_COLUMNS = (
    "id, organisation_id, dataset_id, created_by, status, object_key, "
    "expected_size_bytes, expected_checksum_sha256, expires_at, finalised_at, "
    "created_at, updated_at, manifest"
)


def _to_session(row: dict[str, Any]) -> UploadSession:
    return UploadSession(
        id=row["id"],
        organisation_id=row["organisation_id"],
        dataset_id=row["dataset_id"],
        created_by=row["created_by"],
        status=UploadSessionStatus(row["status"]),
        object_key=row["object_key"],
        expected_size_bytes=row["expected_size_bytes"],
        expected_checksum_sha256=row["expected_checksum_sha256"],
        expires_at=row["expires_at"],
        finalised_at=row["finalised_at"],
        created_at=row["created_at"],
        updated_at=row["updated_at"],
        manifest=row["manifest"] or {},
    )


async def create_upload_session(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    dataset_id: UUID,
    created_by: UUID,
    object_key: str,
    expected_size_bytes: int | None,
    expected_checksum_sha256: str | None,
    expires_at: datetime,
    manifest: dict[str, Any],
) -> UploadSession:
    """Create an upload session in the ``created`` state."""
    cursor = await connection.execute(
        f"""
        INSERT INTO upload_sessions (
            organisation_id, dataset_id, created_by, object_key,
            expected_size_bytes, expected_checksum_sha256, expires_at, manifest
        )
        VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
        RETURNING {_SESSION_COLUMNS}
        """,
        (
            organisation_id,
            dataset_id,
            created_by,
            object_key,
            expected_size_bytes,
            expected_checksum_sha256,
            expires_at,
            Jsonb(manifest),
        ),
    )
    row = await cursor.fetchone()
    assert row is not None
    return _to_session(row)


async def get_upload_session(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    session_id: UUID,
) -> UploadSession | None:
    """Return a session within the organisation, if it exists."""
    cursor = await connection.execute(
        f"""
        SELECT {_SESSION_COLUMNS}
        FROM upload_sessions
        WHERE organisation_id = %s AND id = %s
        """,
        (organisation_id, session_id),
    )
    row = await cursor.fetchone()
    return _to_session(row) if row is not None else None


async def mark_uploaded(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    session_id: UUID,
) -> UploadSession | None:
    """Transition an uploadable session to ``uploaded``."""
    cursor = await connection.execute(
        f"""
        UPDATE upload_sessions
        SET status = 'uploaded', updated_at = now()
        WHERE organisation_id = %s AND id = %s AND status IN ('created', 'uploading')
        RETURNING {_SESSION_COLUMNS}
        """,
        (organisation_id, session_id),
    )
    row = await cursor.fetchone()
    return _to_session(row) if row is not None else None


async def mark_finalising(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    session_id: UUID,
) -> UploadSession | None:
    """Move an uploaded session to ``finalising`` to claim the finalisation."""
    cursor = await connection.execute(
        f"""
        UPDATE upload_sessions
        SET status = 'finalising', updated_at = now()
        WHERE organisation_id = %s AND id = %s AND status = 'uploaded'
        RETURNING {_SESSION_COLUMNS}
        """,
        (organisation_id, session_id),
    )
    row = await cursor.fetchone()
    return _to_session(row) if row is not None else None


async def complete_upload(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    session_id: UUID,
    finalised_at: datetime,
) -> UploadSession | None:
    """Mark a session completed and record its finalisation time."""
    cursor = await connection.execute(
        f"""
        UPDATE upload_sessions
        SET status = 'completed', finalised_at = %s, updated_at = now()
        WHERE organisation_id = %s AND id = %s AND status = 'finalising'
        RETURNING {_SESSION_COLUMNS}
        """,
        (finalised_at, organisation_id, session_id),
    )
    row = await cursor.fetchone()
    return _to_session(row) if row is not None else None


async def fail_upload(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    session_id: UUID,
) -> UploadSession | None:
    """Move a session to the terminal ``failed`` state."""
    cursor = await connection.execute(
        f"""
        UPDATE upload_sessions
        SET status = 'failed', updated_at = now()
        WHERE organisation_id = %s AND id = %s AND status = 'finalising'
        RETURNING {_SESSION_COLUMNS}
        """,
        (organisation_id, session_id),
    )
    row = await cursor.fetchone()
    return _to_session(row) if row is not None else None


async def expire_stale_sessions(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    dataset_id: UUID,
    now: datetime,
) -> int:
    """Expire non-terminal sessions for a dataset whose deadline has passed.

    Returns the number of sessions transitioned so the caller can surface it in
    an audit trail or logs if required.
    """
    cursor = await connection.execute(
        """
        UPDATE upload_sessions
        SET status = 'expired', updated_at = now()
        WHERE organisation_id = %s
          AND dataset_id = %s
          AND status IN ('created', 'uploading', 'uploaded', 'finalising')
          AND expires_at <= %s
        RETURNING id
        """,
        (organisation_id, dataset_id, now),
    )
    rows = await cursor.fetchall()
    return len(rows)


async def claim_expired_sessions(
    connection: _CONNECTION,
    *,
    now: datetime,
    limit: int,
) -> list[UploadSession]:
    """Lock and return a batch of non-terminal sessions past their deadline.

    Crosses all organisations because abandoned-upload cleanup is a maintenance
    operation, not a tenant-scoped request. Rows are locked with
    ``FOR UPDATE SKIP LOCKED`` so concurrent cleanup runs never contend for the
    same sessions.
    """
    cursor = await connection.execute(
        f"""
        SELECT {_SESSION_COLUMNS}
        FROM upload_sessions
        WHERE status IN ('created', 'uploading', 'uploaded', 'finalising')
          AND expires_at <= %s
        ORDER BY expires_at
        LIMIT %s
        FOR UPDATE SKIP LOCKED
        """,
        (now, limit),
    )
    rows = await cursor.fetchall()
    return [_to_session(row) for row in rows]


async def mark_sessions_expired(
    connection: _CONNECTION,
    *,
    session_ids: list[UUID],
) -> int:
    """Transition the given non-terminal sessions to ``expired``.

    Returns the number actually transitioned, which may be lower than the input
    size if a session reached a terminal state concurrently.
    """
    if not session_ids:
        return 0
    cursor = await connection.execute(
        """
        UPDATE upload_sessions
        SET status = 'expired', updated_at = now()
        WHERE id = ANY(%s)
          AND status IN ('created', 'uploading', 'uploaded', 'finalising')
        RETURNING id
        """,
        (session_ids,),
    )
    rows = await cursor.fetchall()
    return len(rows)


__all__ = [
    "claim_expired_sessions",
    "complete_upload",
    "create_upload_session",
    "expire_stale_sessions",
    "fail_upload",
    "get_upload_session",
    "mark_finalising",
    "mark_sessions_expired",
    "mark_uploaded",
]
