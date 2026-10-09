"""Study persistence: explicit SQL and row mapping."""

from __future__ import annotations

from collections.abc import Mapping
from typing import Any
from uuid import UUID

from psycopg import AsyncConnection

from app.core.pagination import Cursor
from app.models.study import Study, StudyStatus

_CONNECTION = AsyncConnection[dict[str, Any]]

_STUDY_COLUMNS = (
    "id, organisation_id, name, description, status, created_by, "
    "created_at, updated_at, archived_at"
)

_UPDATABLE_COLUMNS = frozenset({"name", "description"})


def _escape_like(value: str) -> str:
    """Escape LIKE wildcards so user input is matched literally."""
    return value.replace("\\", "\\\\").replace("%", "\\%").replace("_", "\\_")


def _to_study(row: dict[str, Any]) -> Study:
    return Study(
        id=row["id"],
        organisation_id=row["organisation_id"],
        name=row["name"],
        description=row["description"],
        status=StudyStatus(row["status"]),
        created_by=row["created_by"],
        created_at=row["created_at"],
        updated_at=row["updated_at"],
        archived_at=row["archived_at"],
    )


async def create_study(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    name: str,
    description: str | None,
    created_by: UUID,
) -> Study:
    """Insert a study owned by the organisation."""
    cursor = await connection.execute(
        f"""
        INSERT INTO studies (organisation_id, name, description, created_by)
        VALUES (%s, %s, %s, %s)
        RETURNING {_STUDY_COLUMNS}
        """,
        (organisation_id, name, description, created_by),
    )
    row = await cursor.fetchone()
    assert row is not None
    return _to_study(row)


async def get_study(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    study_id: UUID,
) -> Study | None:
    """Return a study within the given organisation, if it exists."""
    cursor = await connection.execute(
        f"""
        SELECT {_STUDY_COLUMNS}
        FROM studies
        WHERE organisation_id = %s AND id = %s
        """,
        (organisation_id, study_id),
    )
    row = await cursor.fetchone()
    return _to_study(row) if row is not None else None


async def list_studies(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    limit: int,
    cursor: Cursor | None = None,
    status: StudyStatus | None = None,
    search: str | None = None,
) -> list[Study]:
    """List studies for an organisation using keyset pagination.

    Ordering is ``created_at DESC, id DESC``. The organisation filter is always
    applied before the cursor condition. ``search`` performs a case-insensitive
    substring match on the name with LIKE wildcards escaped.
    """
    conditions = ["organisation_id = %s"]
    params: list[Any] = [organisation_id]
    if status is not None:
        conditions.append("status = %s")
        params.append(status.value)
    if search:
        conditions.append("name ILIKE %s ESCAPE '\\'")
        params.append(f"%{_escape_like(search)}%")
    if cursor is not None:
        conditions.append("(created_at, id) < (%s, %s)")
        params.extend([cursor.created_at, cursor.id])
    params.append(limit)
    query = f"""
        SELECT {_STUDY_COLUMNS}
        FROM studies
        WHERE {" AND ".join(conditions)}
        ORDER BY created_at DESC, id DESC
        LIMIT %s
    """
    result = await connection.execute(query, params)
    return [_to_study(row) async for row in result]


async def update_study(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    study_id: UUID,
    changes: Mapping[str, Any],
) -> Study | None:
    """Apply a partial update using only allow-listed columns."""
    assignments: list[str] = []
    params: list[Any] = []
    for column, value in changes.items():
        if column not in _UPDATABLE_COLUMNS:
            raise ValueError(f"column {column!r} is not updatable")
        assignments.append(f"{column} = %s")
        params.append(value)
    if not assignments:
        return await get_study(connection, organisation_id=organisation_id, study_id=study_id)
    assignments.append("updated_at = now()")
    params.extend([organisation_id, study_id])
    cursor = await connection.execute(
        f"""
        UPDATE studies
        SET {", ".join(assignments)}
        WHERE organisation_id = %s AND id = %s
        RETURNING {_STUDY_COLUMNS}
        """,
        params,
    )
    row = await cursor.fetchone()
    return _to_study(row) if row is not None else None


async def archive_study(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    study_id: UUID,
) -> Study | None:
    """Transition a study to the archived state, if it is not already archived."""
    cursor = await connection.execute(
        f"""
        UPDATE studies
        SET status = 'archived',
            archived_at = COALESCE(archived_at, now()),
            updated_at = now()
        WHERE organisation_id = %s AND id = %s AND status = 'active'
        RETURNING {_STUDY_COLUMNS}
        """,
        (organisation_id, study_id),
    )
    row = await cursor.fetchone()
    return _to_study(row) if row is not None else None
