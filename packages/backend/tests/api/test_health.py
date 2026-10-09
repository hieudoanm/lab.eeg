"""API tests for liveness and readiness endpoints."""

from __future__ import annotations

import pytest
from app.main import create_app
from httpx import ASGITransport, AsyncClient

from tests.conftest import make_settings


async def test_liveness_reports_alive(client: AsyncClient) -> None:
    response = await client.get("/health/live")
    assert response.status_code == 200
    body = response.json()
    assert body["status"] == "alive"
    assert body["service"] == "eeg-lab-api"
    assert response.headers["x-request-id"]


async def test_liveness_alias(client: AsyncClient) -> None:
    response = await client.get("/healthz")
    assert response.status_code == 200
    assert response.json()["status"] == "alive"


async def test_readiness_reports_database_ok(client: AsyncClient) -> None:
    response = await client.get("/health/ready")
    assert response.status_code == 200
    body = response.json()
    assert body["status"] == "ready"
    assert body["checks"] == {"database": "ok"}


async def test_readiness_alias(client: AsyncClient) -> None:
    response = await client.get("/readyz")
    assert response.status_code == 200
    assert response.json()["checks"]["database"] == "ok"


@pytest.mark.integration
async def test_readiness_hides_configuration_details(client: AsyncClient) -> None:
    response = await client.get("/health/ready")
    text = response.text.lower()
    assert "password" not in text
    assert "eeg-lab:eeg-lab" not in text
    assert "postgresql://" not in text


@pytest.mark.integration
async def test_readiness_reports_unavailable_when_database_is_down(
    migrated_schema: str,
) -> None:
    settings = make_settings(
        database_url="postgresql://eeg-lab:eeg-lab@127.0.0.1:5999/eeg-lab",
        database_connect_timeout_seconds=0.5,
    )
    app = create_app(settings)
    async with app.router.lifespan_context(app):
        transport = ASGITransport(app=app)
        async with AsyncClient(transport=transport, base_url="http://testserver") as http_client:
            response = await http_client.get("/health/ready")
    assert response.status_code == 503
    body = response.json()
    assert body["status"] == "unavailable"
    assert body["checks"] == {"database": "unavailable"}
    assert "postgresql://" not in response.text


async def test_liveness_does_not_require_database(migrated_schema: str) -> None:
    settings = make_settings(
        database_url="postgresql://eeg-lab:eeg-lab@127.0.0.1:5999/eeg-lab",
        database_connect_timeout_seconds=0.5,
    )
    app = create_app(settings)
    async with app.router.lifespan_context(app):
        transport = ASGITransport(app=app)
        async with AsyncClient(transport=transport, base_url="http://testserver") as http_client:
            response = await http_client.get("/health/live")
    assert response.status_code == 200
    assert response.json()["status"] == "alive"
