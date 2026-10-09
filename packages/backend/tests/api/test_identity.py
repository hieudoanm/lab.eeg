"""API tests for development token issuance and the current-user endpoint."""

from __future__ import annotations

import psycopg
from app.main import create_app
from httpx import ASGITransport, AsyncClient

from tests.conftest import (
    bearer,
    create_membership,
    create_organisation,
    create_user,
    issue_token,
    make_settings,
)


async def test_development_token_issues_usable_token(client: AsyncClient) -> None:
    token = await issue_token(client, subject="ada", display_name="Ada Lovelace")
    response = await client.get("/api/v1/me", headers=bearer(token))
    assert response.status_code == 200
    body = response.json()
    assert body["display_name"] == "Ada Lovelace"
    assert [item["role"] for item in body["memberships"]] == ["owner"]


async def test_development_sign_in_seeds_one_organisation_idempotently(
    client: AsyncClient,
) -> None:
    await issue_token(client, subject="newcomer")
    await issue_token(client, subject="newcomer")
    response = await client.get("/api/v1/me")
    assert response.status_code == 200
    memberships = response.json()["memberships"]
    assert len(memberships) == 1
    assert memberships[0]["organisation_name"] == "Development Organisation"
    assert memberships[0]["role"] == "owner"


async def test_me_requires_authentication(client: AsyncClient) -> None:
    response = await client.get("/api/v1/me")
    assert response.status_code == 401
    assert response.headers["www-authenticate"] == "Bearer"
    assert response.json()["error"]["code"] == "authentication_required"


async def test_me_rejects_invalid_token(client: AsyncClient) -> None:
    response = await client.get("/api/v1/me", headers=bearer("not-a-token"))
    assert response.status_code == 401
    assert response.json()["error"]["code"] == "invalid_token"


async def test_me_rejects_non_bearer_scheme(client: AsyncClient) -> None:
    response = await client.get("/api/v1/me", headers={"Authorization": "Basic abc"})
    assert response.status_code == 401
    assert response.json()["error"]["code"] == "invalid_token"


async def test_token_sets_httponly_session_cookie(client: AsyncClient) -> None:
    response = await client.post("/api/v1/auth/token", json={"subject": "ada"})
    assert response.status_code == 200
    set_cookie = response.headers.get("set-cookie", "")
    assert "eeg-lab_session=" in set_cookie
    assert "httponly" in set_cookie.lower()
    assert "samesite=lax" in set_cookie.lower()


async def test_me_authenticates_from_session_cookie(client: AsyncClient) -> None:
    await client.post(
        "/api/v1/auth/token", json={"subject": "cookie-user", "display_name": "Cookie"}
    )
    response = await client.get("/api/v1/me")
    assert response.status_code == 200
    assert response.json()["display_name"] == "Cookie"


async def test_logout_clears_the_session(client: AsyncClient) -> None:
    await client.post("/api/v1/auth/token", json={"subject": "cookie-user"})
    logout = await client.post("/api/v1/auth/logout")
    assert logout.status_code == 204
    response = await client.get("/api/v1/me")
    assert response.status_code == 401


async def test_development_token_rejects_unknown_fields(client: AsyncClient) -> None:
    response = await client.post(
        "/api/v1/auth/token", json={"subject": "ada", "unexpected": "value"}
    )
    assert response.status_code == 422
    assert response.json()["error"]["code"] == "validation_error"


async def test_me_lists_memberships_without_provider_internals(
    client: AsyncClient, db: psycopg.Connection[dict[str, object]]
) -> None:
    user_id = create_user(db, subject="ada", display_name="Ada")
    organisation_id = create_organisation(db, name="Acme")
    create_membership(db, organisation_id=organisation_id, user_id=user_id, role="owner")

    token = await issue_token(client, subject="ada")
    response = await client.get("/api/v1/me", headers=bearer(token))
    assert response.status_code == 200
    body = response.json()
    assert [item["organisation_name"] for item in body["memberships"]] == ["Acme"]
    assert body["memberships"][0]["role"] == "owner"
    assert "external_subject" not in body
    assert "identity_provider" not in body


async def test_disabled_user_is_rejected(
    client: AsyncClient, db: psycopg.Connection[dict[str, object]]
) -> None:
    user_id = create_user(db, subject="blocked")
    db.execute("UPDATE users SET disabled_at = now() WHERE id = %s", (user_id,))
    token = await issue_token(client, subject="blocked")
    response = await client.get("/api/v1/me", headers=bearer(token))
    assert response.status_code == 403
    assert response.json()["error"]["code"] == "account_disabled"


async def test_development_token_endpoint_is_hidden_when_disabled() -> None:
    app = create_app(make_settings(auth_mode="oidc"))
    async with app.router.lifespan_context(app):
        transport = ASGITransport(app=app)
        async with AsyncClient(transport=transport, base_url="http://testserver") as http:
            response = await http.post("/api/v1/auth/token", json={"subject": "ada"})
    assert response.status_code == 404
