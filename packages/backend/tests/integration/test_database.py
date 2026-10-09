"""Integration tests for the asynchronous database pool wrapper."""

from __future__ import annotations

from typing import Any

import psycopg
import pytest
from app.core.exceptions import ServiceUnavailableError
from app.db.pool import Database

from tests.conftest import make_settings

pytestmark = pytest.mark.integration


async def test_ping_returns_true_when_database_answers(database: Database) -> None:
    assert await database.ping() is True


async def test_connection_yields_mapping_rows(database: Database) -> None:
    async with database.connection() as connection:
        cursor = await connection.execute("SELECT 1 AS value")
        row = await cursor.fetchone()
    assert row == {"value": 1}


async def test_transaction_commits_on_success(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    async with database.transaction() as connection:
        await connection.execute("INSERT INTO organisations (name) VALUES ('committed')")

    row = db.execute("SELECT count(*) AS n FROM organisations").fetchone()
    assert row is not None
    assert row["n"] == 1


async def test_transaction_rolls_back_on_error(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    with pytest.raises(RuntimeError):
        async with database.transaction() as connection:
            await connection.execute("INSERT INTO organisations (name) VALUES ('discarded')")
            raise RuntimeError("boom")

    row = db.execute("SELECT count(*) AS n FROM organisations").fetchone()
    assert row is not None
    assert row["n"] == 0


async def test_unreachable_database_raises_service_unavailable(migrated_schema: str) -> None:
    instance = Database(
        make_settings(
            database_url="postgresql://eeg-lab:eeg-lab@127.0.0.1:5999/eeg-lab",
            database_connect_timeout_seconds=0.5,
        )
    )
    await instance.connect()
    with pytest.raises(ServiceUnavailableError):
        async with instance.connection():
            pass
    await instance.close()
