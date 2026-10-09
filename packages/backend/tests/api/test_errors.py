"""API tests for the consistent error envelope."""

from __future__ import annotations

from app.main import create_app
from httpx import ASGITransport, AsyncClient

from tests.conftest import make_settings


async def test_unknown_route_returns_not_found_envelope(client: AsyncClient) -> None:
    response = await client.get("/api/v1/does-not-exist")
    assert response.status_code == 404
    error = response.json()["error"]
    assert error["code"] == "not_found"
    assert error["message"]
    assert error["request_id"]
    assert response.headers["x-request-id"] == error["request_id"]


async def test_wrong_method_returns_method_not_allowed(client: AsyncClient) -> None:
    response = await client.post("/health/live")
    assert response.status_code == 405
    assert response.json()["error"]["code"] == "method_not_allowed"


async def test_request_identifier_is_echoed(client: AsyncClient) -> None:
    response = await client.get("/health/live", headers={"X-Request-Id": "caller-123"})
    assert response.headers["x-request-id"] == "caller-123"
    assert response.status_code == 200


async def test_unsafe_request_identifier_is_not_reflected(client: AsyncClient) -> None:
    response = await client.get("/health/live", headers={"X-Request-Id": "bad id"})
    assert response.headers["x-request-id"] != "bad id"


async def test_oversized_request_body_is_rejected(migrated_schema: str) -> None:
    app = create_app(make_settings(max_request_body_bytes=16))
    async with app.router.lifespan_context(app):
        transport = ASGITransport(app=app)
        async with AsyncClient(transport=transport, base_url="http://testserver") as http_client:
            response = await http_client.post("/api/v1/anything", content=b"x" * 64)
    assert response.status_code == 413
    error = response.json()["error"]
    assert error["code"] == "payload_too_large"
    assert error["request_id"]


async def test_error_details_are_omitted_when_empty(client: AsyncClient) -> None:
    response = await client.get("/api/v1/does-not-exist")
    assert response.json()["error"].get("details", []) == []
