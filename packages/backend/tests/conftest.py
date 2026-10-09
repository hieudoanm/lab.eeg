"""Shared pytest fixtures.

The suite runs against a real PostgreSQL database. ``DATABASE_URL`` must point
at an isolated test database; the migration fixture recreates the schema before
the session so tests never depend on leftover state.
"""

from __future__ import annotations

import os
import tempfile
from collections.abc import AsyncIterator, Iterator
from typing import Any, cast
from uuid import UUID

import psycopg
import pytest
from httpx import ASGITransport, AsyncClient

TEST_DATABASE_URL = os.environ.get(
    "TEST_DATABASE_URL",
    "postgresql://eeg-lab:eeg-lab@localhost:5432/eeg-lab_test",
)
TEST_STORAGE_DIR = tempfile.mkdtemp(prefix="eeg-lab-test-uploads-")

# Force migrations and settings to use the test database.
os.environ["DATABASE_URL"] = TEST_DATABASE_URL
os.environ.setdefault("APP_ENV", "test")

TRUNCATE_TABLES = (
    "audit_events",
    "idempotency_keys",
    "outbox_events",
    "analysis_results",
    "analysis_jobs",
    "upload_sessions",
    "datasets",
    "studies",
    "memberships",
    "organisations",
    "users",
)

from app.core.config import Settings, clear_settings_cache  # noqa: E402
from app.db.pool import Database  # noqa: E402
from app.main import create_app  # noqa: E402


def make_settings(**overrides: object) -> Settings:
    """Build test settings without reading the developer's environment."""
    values: dict[str, object] = {
        "app_env": "test",
        "app_name": "eeg-lab-api",
        "app_version": "0.1.0",
        "log_level": "WARNING",
        "log_format": "console",
        "database_url": TEST_DATABASE_URL,
        "database_pool_min_size": 1,
        "database_pool_max_size": 4,
        "auth_mode": "development",
        "local_storage_path": TEST_STORAGE_DIR,
    }
    values.update(overrides)
    return Settings(**values)  # type: ignore[arg-type]


@pytest.fixture(scope="session")
def migrated_database() -> Iterator[str]:
    """Recreate the test schema from migrations once per session."""
    clear_settings_cache()
    os.environ["DATABASE_URL"] = TEST_DATABASE_URL
    with psycopg.connect(TEST_DATABASE_URL, autocommit=True) as connection:
        connection.execute("DROP SCHEMA IF EXISTS public CASCADE")
        connection.execute("CREATE SCHEMA public")
    from app.db.migrations import upgrade

    upgrade(TEST_DATABASE_URL)
    yield TEST_DATABASE_URL
    clear_settings_cache()


@pytest.fixture
def migrated_schema(migrated_database: str) -> Iterator[str]:
    """Provide a migrated database with all rows removed before each test."""
    _truncate()
    yield migrated_database


def _truncate() -> None:
    with psycopg.connect(TEST_DATABASE_URL, autocommit=True) as connection:
        connection.execute(f"TRUNCATE {', '.join(TRUNCATE_TABLES)} RESTART IDENTITY CASCADE")


@pytest.fixture
def db(migrated_schema: str) -> Iterator[psycopg.Connection[dict[str, Any]]]:
    """Yield a dict-row connection with autocommit for constraint tests."""
    from psycopg.rows import dict_row

    with psycopg.connect(TEST_DATABASE_URL, autocommit=True, row_factory=dict_row) as connection:
        yield connection


@pytest.fixture
async def database(migrated_schema: str) -> AsyncIterator[Database]:
    """A connected pool bound to the test database."""
    instance = Database(make_settings())
    await instance.connect()
    try:
        yield instance
    finally:
        await instance.close()


@pytest.fixture
async def client(migrated_schema: str) -> AsyncIterator[AsyncClient]:
    """An HTTPX client bound to the application with lifespan managed."""
    app = create_app(make_settings())
    async with app.router.lifespan_context(app):
        transport = ASGITransport(app=app)
        async with AsyncClient(transport=transport, base_url="http://testserver") as http_client:
            yield http_client


# ---------------------------------------------------------------------------
# Seed helpers (synchronous, use with the ``db`` fixture)
# ---------------------------------------------------------------------------


def create_user(
    connection: psycopg.Connection[dict[str, Any]],
    *,
    subject: str,
    identity_provider: str = "local",
    display_name: str | None = None,
    email: str | None = None,
) -> UUID:
    """Insert a user and return its identifier."""
    row = connection.execute(
        """
        INSERT INTO users (identity_provider, external_subject, display_name, email)
        VALUES (%s, %s, %s, %s)
        RETURNING id
        """,
        (identity_provider, subject, display_name, email),
    ).fetchone()
    assert row is not None
    return cast(UUID, row["id"])


def create_organisation(
    connection: psycopg.Connection[dict[str, Any]],
    *,
    name: str,
    slug: str | None = None,
) -> UUID:
    """Insert an organisation and return its identifier."""
    row = connection.execute(
        "INSERT INTO organisations (name, slug) VALUES (%s, %s) RETURNING id",
        (name, slug),
    ).fetchone()
    assert row is not None
    return cast(UUID, row["id"])


def create_membership(
    connection: psycopg.Connection[dict[str, Any]],
    *,
    organisation_id: UUID,
    user_id: UUID,
    role: str = "owner",
) -> UUID:
    """Insert a membership and return its identifier."""
    row = connection.execute(
        """
        INSERT INTO memberships (organisation_id, user_id, role)
        VALUES (%s, %s, %s)
        RETURNING id
        """,
        (organisation_id, user_id, role),
    ).fetchone()
    assert row is not None
    return cast(UUID, row["id"])


async def issue_token(
    client: AsyncClient,
    *,
    subject: str = "demo-researcher",
    identity_provider: str = "local",
    **extra: str,
) -> str:
    """Mint a development access token through the API and return it."""
    payload: dict[str, str] = {
        "subject": subject,
        "identity_provider": identity_provider,
        **extra,
    }
    response = await client.post("/api/v1/auth/token", json=payload)
    assert response.status_code == 200, response.text
    token = response.json()["access_token"]
    assert isinstance(token, str)
    return token


def bearer(token: str) -> dict[str, str]:
    """Build an Authorization header for a bearer token."""
    return {"Authorization": f"Bearer {token}"}
