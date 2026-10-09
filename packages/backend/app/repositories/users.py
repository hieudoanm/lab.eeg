"""User persistence: identity lookup and just-in-time provisioning."""

from __future__ import annotations

from typing import Any
from uuid import UUID

from psycopg import AsyncConnection

from app.models.identity import User

_CONNECTION = AsyncConnection[dict[str, Any]]

_USER_COLUMNS = "id, identity_provider, external_subject, email, display_name, disabled_at"


def _to_user(row: dict[str, Any]) -> User:
    return User(
        id=row["id"],
        identity_provider=row["identity_provider"],
        external_subject=row["external_subject"],
        email=row["email"],
        display_name=row["display_name"],
        disabled_at=row["disabled_at"],
    )


async def get_user_by_identity(
    connection: _CONNECTION,
    *,
    identity_provider: str,
    external_subject: str,
) -> User | None:
    """Return the user mapped to an identity provider subject, if any."""
    cursor = await connection.execute(
        f"""
        SELECT {_USER_COLUMNS}
        FROM users
        WHERE identity_provider = %s AND external_subject = %s
        """,
        (identity_provider, external_subject),
    )
    row = await cursor.fetchone()
    return _to_user(row) if row is not None else None


async def get_user(connection: _CONNECTION, user_id: UUID) -> User | None:
    """Return a user by primary key."""
    cursor = await connection.execute(
        f"SELECT {_USER_COLUMNS} FROM users WHERE id = %s",
        (user_id,),
    )
    row = await cursor.fetchone()
    return _to_user(row) if row is not None else None


async def upsert_user(
    connection: _CONNECTION,
    *,
    identity_provider: str,
    external_subject: str,
    display_name: str | None = None,
    email: str | None = None,
) -> User:
    """Create a user for a subject or refresh their profile fields.

    Profile fields are only overwritten when a value is supplied, so claims that
    omit a display name or email never erase stored data.
    """
    cursor = await connection.execute(
        f"""
        INSERT INTO users (identity_provider, external_subject, display_name, email)
        VALUES (%s, %s, %s, %s)
        ON CONFLICT (identity_provider, external_subject) DO UPDATE
            SET display_name = COALESCE(EXCLUDED.display_name, users.display_name),
                email = COALESCE(EXCLUDED.email, users.email),
                updated_at = now()
        RETURNING {_USER_COLUMNS}
        """,
        (identity_provider, external_subject, display_name, email),
    )
    row = await cursor.fetchone()
    assert row is not None
    return _to_user(row)
