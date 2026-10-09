"""Organisation persistence and tenant-scoped listing."""

from __future__ import annotations

from typing import Any
from uuid import UUID

from psycopg import AsyncConnection

from app.models.identity import Membership, Organisation, Role

_CONNECTION = AsyncConnection[dict[str, Any]]

_ORGANISATION_COLUMNS = "id, name, slug, status, created_at, updated_at"
_MEMBERSHIP_COLUMNS = "id, organisation_id, user_id, role, created_at, updated_at"


def _to_organisation(row: dict[str, Any]) -> Organisation:
    return Organisation(
        id=row["id"],
        name=row["name"],
        slug=row["slug"],
        status=row["status"],
        created_at=row["created_at"],
        updated_at=row["updated_at"],
    )


def _to_membership(row: dict[str, Any]) -> Membership:
    return Membership(
        id=row["id"],
        organisation_id=row["organisation_id"],
        user_id=row["user_id"],
        role=Role(row["role"]),
        created_at=row["created_at"],
        updated_at=row["updated_at"],
    )


async def get_organisation(connection: _CONNECTION, organisation_id: UUID) -> Organisation | None:
    """Return an organisation by primary key."""
    cursor = await connection.execute(
        f"SELECT {_ORGANISATION_COLUMNS} FROM organisations WHERE id = %s",
        (organisation_id,),
    )
    row = await cursor.fetchone()
    return _to_organisation(row) if row is not None else None


async def create_organisation(
    connection: _CONNECTION,
    *,
    name: str,
    slug: str | None = None,
) -> Organisation:
    """Insert a new organisation."""
    cursor = await connection.execute(
        f"""
        INSERT INTO organisations (name, slug)
        VALUES (%s, %s)
        RETURNING {_ORGANISATION_COLUMNS}
        """,
        (name, slug),
    )
    row = await cursor.fetchone()
    assert row is not None
    return _to_organisation(row)


async def list_organisations_for_user(
    connection: _CONNECTION, user_id: UUID
) -> list[tuple[Organisation, Membership]]:
    """Return every organisation the user belongs to, with their membership.

    Ordering is deterministic by organisation name then id so membership lists
    are stable across requests.
    """
    cursor = await connection.execute(
        """
        SELECT o.id, o.name, o.slug, o.status, o.created_at, o.updated_at,
               m.id AS membership_id, m.organisation_id AS membership_organisation_id,
               m.user_id AS membership_user_id, m.role AS membership_role,
               m.created_at AS membership_created_at, m.updated_at AS membership_updated_at
        FROM organisations AS o
        JOIN memberships AS m ON m.organisation_id = o.id
        WHERE m.user_id = %s
        ORDER BY o.name ASC, o.id ASC
        """,
        (user_id,),
    )
    results: list[tuple[Organisation, Membership]] = []
    async for row in cursor:
        organisation = _to_organisation(row)
        membership = Membership(
            id=row["membership_id"],
            organisation_id=row["membership_organisation_id"],
            user_id=row["membership_user_id"],
            role=Role(row["membership_role"]),
            created_at=row["membership_created_at"],
            updated_at=row["membership_updated_at"],
        )
        results.append((organisation, membership))
    return results
