"""API tests for organisation detail and membership management endpoints."""

from __future__ import annotations

from typing import Any, cast
from uuid import UUID, uuid4

import psycopg
from httpx import AsyncClient

from tests.conftest import (
    bearer,
    create_membership,
    create_organisation,
    create_user,
    issue_token,
)

BASE = "/api/v1/organisations/{organisation_id}"
MEMBERSHIPS = "/api/v1/organisations/{organisation_id}/memberships"
MEMBERSHIP = "/api/v1/organisations/{organisation_id}/memberships/{membership_id}"


def _seed_owner(
    db: psycopg.Connection[dict[str, Any]],
    *,
    subject: str = "owner",
    organisation_name: str = "Research Organisation",
) -> tuple[UUID, UUID]:
    user_id = create_user(db, subject=subject)
    organisation_id = create_organisation(db, name=organisation_name)
    create_membership(db, organisation_id=organisation_id, user_id=user_id, role="owner")
    return user_id, organisation_id


def _add_member(
    db: psycopg.Connection[dict[str, Any]],
    *,
    organisation_id: UUID,
    subject: str,
    role: str,
    display_name: str | None = None,
    email: str | None = None,
) -> UUID:
    user_id = create_user(db, subject=subject, display_name=display_name, email=email)
    create_membership(db, organisation_id=organisation_id, user_id=user_id, role=role)
    return user_id


def _membership_id(
    db: psycopg.Connection[dict[str, Any]], organisation_id: UUID, user_id: UUID
) -> UUID:
    row = db.execute(
        "SELECT id FROM memberships WHERE organisation_id = %s AND user_id = %s",
        (organisation_id, user_id),
    ).fetchone()
    assert row is not None
    return cast(UUID, row["id"])


def _action_count(db: psycopg.Connection[dict[str, Any]], action: str) -> int:
    row = db.execute(
        "SELECT COUNT(*) AS count FROM audit_events WHERE action = %s", (action,)
    ).fetchone()
    assert row is not None
    return int(row["count"])


async def test_read_organisation_returns_detail(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, organisation_name="Science Lab")
    token = await issue_token(client, subject="owner")

    response = await client.get(BASE.format(organisation_id=organisation_id), headers=bearer(token))

    assert response.status_code == 200, response.text
    body = response.json()
    assert body["name"] == "Science Lab"
    assert body["role"] == "owner"
    assert body["slug"] is None
    assert body["created_at"].endswith("Z")


async def test_read_organisation_is_hidden_from_non_members(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, subject="owner-a")
    _seed_owner(db, subject="owner-b", organisation_name="Other")
    token = await issue_token(client, subject="owner-b")

    response = await client.get(BASE.format(organisation_id=organisation_id), headers=bearer(token))

    assert response.status_code == 404
    assert response.json()["error"]["code"] == "not_found"


async def test_list_memberships_returns_member_profiles(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, subject="owner")
    _add_member(
        db,
        organisation_id=organisation_id,
        subject="viewer",
        role="viewer",
        display_name="Vera",
        email="vera@example.org",
    )
    token = await issue_token(client, subject="owner")

    response = await client.get(
        MEMBERSHIPS.format(organisation_id=organisation_id), headers=bearer(token)
    )

    assert response.status_code == 200, response.text
    items = response.json()["items"]
    assert [item["role"] for item in items] == ["owner", "viewer"]
    assert items[1]["display_name"] == "Vera"
    assert items[1]["email"] == "vera@example.org"
    assert "user_id" in items[0]
    assert response.json()["has_more"] is False


async def test_add_membership_provisions_a_new_user(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, subject="owner")
    token = await issue_token(client, subject="owner")

    response = await client.post(
        MEMBERSHIPS.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={
            "identity_provider": "local",
            "subject": "newcomer",
            "role": "researcher",
            "display_name": "New Comer",
            "email": "new@example.org",
        },
    )

    assert response.status_code == 201, response.text
    body = response.json()
    assert body["role"] == "researcher"
    assert body["display_name"] == "New Comer"
    assert body["email"] == "new@example.org"
    assert _action_count(db, "membership.created") == 1
    listing = await client.get(
        MEMBERSHIPS.format(organisation_id=organisation_id), headers=bearer(token)
    )
    assert len(listing.json()["items"]) == 2


async def test_add_membership_reuses_an_existing_active_user(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, subject="owner")
    existing_id = create_user(db, subject="existing", display_name="Existing")
    token = await issue_token(client, subject="owner")

    response = await client.post(
        MEMBERSHIPS.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={
            "identity_provider": "local",
            "subject": "existing",
            "role": "viewer",
        },
    )

    assert response.status_code == 201, response.text
    assert response.json()["user_id"] == str(existing_id)


async def test_add_membership_rejects_a_duplicate_member(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, subject="owner")
    token = await issue_token(client, subject="owner")
    create_user(db, subject="duplicate")
    payload = {"identity_provider": "local", "subject": "duplicate", "role": "viewer"}
    first = await client.post(
        MEMBERSHIPS.format(organisation_id=organisation_id),
        headers=bearer(token),
        json=payload,
    )
    assert first.status_code == 201

    second = await client.post(
        MEMBERSHIPS.format(organisation_id=organisation_id),
        headers=bearer(token),
        json=payload,
    )

    assert second.status_code == 409
    assert second.json()["error"]["code"] == "membership_exists"


async def test_add_membership_rejects_a_disabled_user(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, subject="owner")
    create_user(db, subject="blocked")
    db.execute("UPDATE users SET disabled_at = now() WHERE external_subject = 'blocked'")
    token = await issue_token(client, subject="owner")

    response = await client.post(
        MEMBERSHIPS.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={"identity_provider": "local", "subject": "blocked", "role": "viewer"},
    )

    assert response.status_code == 409
    assert response.json()["error"]["code"] == "user_disabled"


async def test_add_membership_validates_the_payload(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, subject="owner")
    token = await issue_token(client, subject="owner")

    response = await client.post(
        MEMBERSHIPS.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={"identity_provider": "local", "role": "viewer"},
    )

    assert response.status_code == 422
    assert response.json()["error"]["code"] == "validation_error"


async def test_viewer_cannot_add_membership(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, subject="owner")
    _add_member(db, organisation_id=organisation_id, subject="viewer", role="viewer")
    token = await issue_token(client, subject="viewer")

    response = await client.post(
        MEMBERSHIPS.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={"identity_provider": "local", "subject": "anyone", "role": "viewer"},
    )

    assert response.status_code == 403
    assert response.json()["error"]["code"] == "insufficient_role"


async def test_change_membership_role_updates_the_member(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, subject="owner")
    member_id = _add_member(
        db, organisation_id=organisation_id, subject="member", role="researcher"
    )
    membership_id = _membership_id(db, organisation_id, member_id)
    token = await issue_token(client, subject="owner")

    response = await client.patch(
        MEMBERSHIP.format(organisation_id=organisation_id, membership_id=membership_id),
        headers=bearer(token),
        json={"role": "viewer"},
    )

    assert response.status_code == 200, response.text
    assert response.json()["role"] == "viewer"
    assert _action_count(db, "membership.role_changed") == 1


async def test_change_membership_role_is_a_no_op_when_unchanged(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, subject="owner")
    member_id = _add_member(db, organisation_id=organisation_id, subject="member", role="viewer")
    membership_id = _membership_id(db, organisation_id, member_id)
    token = await issue_token(client, subject="owner")

    response = await client.patch(
        MEMBERSHIP.format(organisation_id=organisation_id, membership_id=membership_id),
        headers=bearer(token),
        json={"role": "viewer"},
    )

    assert response.status_code == 200
    assert response.json()["role"] == "viewer"
    assert _action_count(db, "membership.role_changed") == 0


async def test_change_membership_role_refuses_to_demote_the_last_owner(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    owner_id, organisation_id = _seed_owner(db, subject="owner")
    membership_id = _membership_id(db, organisation_id, owner_id)
    token = await issue_token(client, subject="owner")

    response = await client.patch(
        MEMBERSHIP.format(organisation_id=organisation_id, membership_id=membership_id),
        headers=bearer(token),
        json={"role": "viewer"},
    )

    assert response.status_code == 409
    assert response.json()["error"]["code"] == "last_owner"


async def test_change_membership_role_rejects_an_unknown_membership(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, subject="owner")
    token = await issue_token(client, subject="owner")

    response = await client.patch(
        MEMBERSHIP.format(organisation_id=organisation_id, membership_id=uuid4()),
        headers=bearer(token),
        json={"role": "viewer"},
    )

    assert response.status_code == 404


async def test_viewer_cannot_change_membership_role(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, subject="owner")
    viewer_id = _add_member(db, organisation_id=organisation_id, subject="viewer", role="viewer")
    membership_id = _membership_id(db, organisation_id, viewer_id)
    # The viewer attempts to change their own role.
    viewer_token = await issue_token(client, subject="viewer")

    response = await client.patch(
        MEMBERSHIP.format(organisation_id=organisation_id, membership_id=membership_id),
        headers=bearer(viewer_token),
        json={"role": "owner"},
    )

    assert response.status_code == 403


async def test_remove_membership_deletes_the_member(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, subject="owner")
    member_id = _add_member(db, organisation_id=organisation_id, subject="member", role="viewer")
    membership_id = _membership_id(db, organisation_id, member_id)
    token = await issue_token(client, subject="owner")

    response = await client.delete(
        MEMBERSHIP.format(organisation_id=organisation_id, membership_id=membership_id),
        headers=bearer(token),
    )

    assert response.status_code == 204
    assert _action_count(db, "membership.removed") == 1
    listing = await client.get(
        MEMBERSHIPS.format(organisation_id=organisation_id), headers=bearer(token)
    )
    assert len(listing.json()["items"]) == 1


async def test_remove_membership_refuses_to_remove_the_last_owner(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    owner_id, organisation_id = _seed_owner(db, subject="owner")
    membership_id = _membership_id(db, organisation_id, owner_id)
    token = await issue_token(client, subject="owner")

    response = await client.delete(
        MEMBERSHIP.format(organisation_id=organisation_id, membership_id=membership_id),
        headers=bearer(token),
    )

    assert response.status_code == 409
    assert response.json()["error"]["code"] == "last_owner"


async def test_remove_membership_rejects_an_unknown_membership(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, subject="owner")
    token = await issue_token(client, subject="owner")

    response = await client.delete(
        MEMBERSHIP.format(organisation_id=organisation_id, membership_id=uuid4()),
        headers=bearer(token),
    )

    assert response.status_code == 404


async def test_viewer_cannot_remove_membership(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_owner(db, subject="owner")
    viewer_id = _add_member(db, organisation_id=organisation_id, subject="viewer", role="viewer")
    membership_id = _membership_id(db, organisation_id, viewer_id)
    viewer_token = await issue_token(client, subject="viewer")

    response = await client.delete(
        MEMBERSHIP.format(organisation_id=organisation_id, membership_id=membership_id),
        headers=bearer(viewer_token),
    )

    assert response.status_code == 403


async def test_memberships_are_scoped_to_the_caller_organisation(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_a = _seed_owner(db, subject="owner-a", organisation_name="A")
    _seed_owner(db, subject="owner-b", organisation_name="B")
    token_b = await issue_token(client, subject="owner-b")

    response = await client.get(
        MEMBERSHIPS.format(organisation_id=organisation_a), headers=bearer(token_b)
    )

    assert response.status_code == 404
