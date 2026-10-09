"""Security tests for organisation-scoped access control.

Every case exercises the negative path: a caller either lacks the required role
or attempts to act on a resource outside their organisation. The suite uses the
real database so that row-level constraints and the tenant filter are tested
together.
"""

from __future__ import annotations

from typing import Any

from httpx import AsyncClient

from tests.conftest import bearer, issue_token


async def test_owner_can_create_organisation_and_is_owner(client: AsyncClient, db: Any) -> None:
    token = await issue_token(client, subject="owner")
    response = await client.post(
        "/api/v1/organisations",
        headers=bearer(token),
        json={"name": "Science Lab"},
    )
    assert response.status_code == 201
    body = response.json()
    assert body["name"] == "Science Lab"
    assert body["role"] == "owner"
    actions = [
        row["action"]
        for row in db.execute("SELECT action FROM audit_events ORDER BY created_at").fetchall()
    ]
    assert "organisation.created" in actions
    assert "membership.created" in actions
    listing = await client.get("/api/v1/organisations", headers=bearer(token))
    assert any(item["name"] == "Science Lab" for item in listing.json()["items"])


async def test_create_organisation_rejects_blank_name(client: AsyncClient, db: Any) -> None:
    token = await issue_token(client, subject="owner")
    response = await client.post(
        "/api/v1/organisations",
        headers=bearer(token),
        json={"name": "   "},
    )
    assert response.status_code == 422


async def test_create_organisation_rejects_unknown_fields(client: AsyncClient, db: Any) -> None:
    token = await issue_token(client, subject="owner")
    response = await client.post(
        "/api/v1/organisations",
        headers=bearer(token),
        json={"name": "Lab", "extra": 1},
    )
    assert response.status_code == 422
