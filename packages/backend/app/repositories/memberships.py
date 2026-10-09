"""Membership persistence and organisation-scoped row locking."""

from __future__ import annotations

from typing import Any
from uuid import UUID

from psycopg import AsyncConnection, errors

from app.core.exceptions import ConflictError
from app.models.identity import Membership, MembershipView, Role

_CONNECTION = AsyncConnection[dict[str, Any]]

_MEMBERSHIP_COLUMNS = "id, organisation_id, user_id, role, created_at, updated_at"


def _to_membership(row: dict[str, Any]) -> Membership:
    return Membership(
        id=row["id"],
        organisation_id=row["organisation_id"],
        user_id=row["user_id"],
        role=Role(row["role"]),
        created_at=row["created_at"],
        updated_at=row["updated_at"],
    )


async def lock_memberships(connection: _CONNECTION, organisation_id: UUID) -> None:
    """Lock every membership row in an organisation for the transaction.

    Membership mutations acquire this lock so that the last-owner invariant is
    evaluated against a stable set of rows under concurrent requests.
    """
    await connection.execute(
        "SELECT id FROM memberships WHERE organisation_id = %s FOR UPDATE",
        (organisation_id,),
    )


async def get_membership(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    user_id: UUID,
) -> Membership | None:
    cursor = await connection.execute(
        f"""
        SELECT {_MEMBERSHIP_COLUMNS}
        FROM memberships
        WHERE organisation_id = %s AND user_id = %s
        """,
        (organisation_id, user_id),
    )
    row = await cursor.fetchone()
    return _to_membership(row) if row is not None else None


async def get_membership_by_id(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    membership_id: UUID,
) -> Membership | None:
    cursor = await connection.execute(
        f"""
        SELECT {_MEMBERSHIP_COLUMNS}
        FROM memberships
        WHERE organisation_id = %s AND id = %s
        """,
        (organisation_id, membership_id),
    )
    row = await cursor.fetchone()
    return _to_membership(row) if row is not None else None


async def list_memberships(connection: _CONNECTION, organisation_id: UUID) -> list[MembershipView]:
    """List memberships with member profile fields, ordered deterministically."""
    cursor = await connection.execute(
        """
        SELECT m.id, m.organisation_id, m.user_id, m.role, m.created_at, m.updated_at,
               u.display_name AS user_display_name, u.email AS user_email
        FROM memberships AS m
        JOIN users AS u ON u.id = m.user_id
        WHERE m.organisation_id = %s
        ORDER BY m.created_at ASC, m.id ASC
        """,
        (organisation_id,),
    )
    views: list[MembershipView] = []
    async for row in cursor:
        views.append(
            MembershipView(
                membership=_to_membership(row),
                display_name=row["user_display_name"],
                email=row["user_email"],
            )
        )
    return views


async def count_owners(connection: _CONNECTION, organisation_id: UUID) -> int:
    """Count active owners (owners whose user account is not disabled)."""
    cursor = await connection.execute(
        """
        SELECT count(*) AS total
        FROM memberships AS m
        JOIN users AS u ON u.id = m.user_id
        WHERE m.organisation_id = %s AND m.role = 'owner' AND u.disabled_at IS NULL
        """,
        (organisation_id,),
    )
    row = await cursor.fetchone()
    return int(row["total"]) if row is not None else 0


async def create_membership(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    user_id: UUID,
    role: Role,
) -> Membership:
    try:
        cursor = await connection.execute(
            f"""
            INSERT INTO memberships (organisation_id, user_id, role)
            VALUES (%s, %s, %s)
            RETURNING {_MEMBERSHIP_COLUMNS}
            """,
            (organisation_id, user_id, role.value),
        )
    except errors.UniqueViolation as exc:
        raise ConflictError(
            "The user is already a member of this organisation.",
            code="membership_exists",
        ) from exc
    row = await cursor.fetchone()
    assert row is not None
    return _to_membership(row)


async def update_membership_role(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    membership_id: UUID,
    role: Role,
) -> Membership | None:
    cursor = await connection.execute(
        f"""
        UPDATE memberships
        SET role = %s, updated_at = now()
        WHERE organisation_id = %s AND id = %s
        RETURNING {_MEMBERSHIP_COLUMNS}
        """,
        (role.value, organisation_id, membership_id),
    )
    row = await cursor.fetchone()
    return _to_membership(row) if row is not None else None


async def delete_membership(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    membership_id: UUID,
) -> Membership | None:
    cursor = await connection.execute(
        f"""
        DELETE FROM memberships
        WHERE organisation_id = %s AND id = %s
        RETURNING {_MEMBERSHIP_COLUMNS}
        """,
        (organisation_id, membership_id),
    )
    row = await cursor.fetchone()
    return _to_membership(row) if row is not None else None
