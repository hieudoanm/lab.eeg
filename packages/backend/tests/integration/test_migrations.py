"""Integration tests for SQL migrations against a real PostgreSQL instance."""

from __future__ import annotations

from pathlib import Path

import psycopg
import pytest
from app.core.config import clear_settings_cache
from app.db.migrations import downgrade, upgrade

from tests.conftest import TEST_DATABASE_URL

CORE_TABLES = {
    "users",
    "organisations",
    "memberships",
    "studies",
    "datasets",
    "upload_sessions",
    "analysis_jobs",
    "analysis_results",
    "outbox_events",
    "idempotency_keys",
    "audit_events",
}


def _tables(url: str) -> set[str]:
    with psycopg.connect(url) as connection:
        rows = connection.execute(
            "SELECT tablename FROM pg_tables WHERE schemaname = 'public'"
        ).fetchall()
    return {row[0] for row in rows}


@pytest.mark.integration
def test_all_core_tables_exist(migrated_database: str) -> None:
    tables = _tables(migrated_database)
    assert tables >= CORE_TABLES
    assert "schema_migrations" in tables


@pytest.mark.integration
def test_migrations_round_trip_from_empty_database(monkeypatch: pytest.MonkeyPatch) -> None:
    admin_url = TEST_DATABASE_URL.rsplit("/", 1)[0] + "/postgres"
    target_db = "eeg-lab_migration_roundtrip"
    target_url = TEST_DATABASE_URL.rsplit("/", 1)[0] + f"/{target_db}"

    with psycopg.connect(admin_url, autocommit=True) as admin:
        admin.execute(f'DROP DATABASE IF EXISTS "{target_db}" WITH (FORCE)')
        admin.execute(f'CREATE DATABASE "{target_db}"')

    monkeypatch.setenv("DATABASE_URL", target_url)
    clear_settings_cache()
    try:
        assert upgrade(target_url) == [
            "0001_initial_schema",
            "0002_add_upload_sessions_manifest",
        ]
        upgraded = _tables(target_url)
        assert upgraded >= CORE_TABLES

        assert downgrade(target_url) == [
            "0002_add_upload_sessions_manifest",
            "0001_initial_schema",
        ]
        remaining = _tables(target_url)
        assert not (CORE_TABLES & remaining)
    finally:
        clear_settings_cache()
        with psycopg.connect(admin_url, autocommit=True) as admin:
            admin.execute(f'DROP DATABASE IF EXISTS "{target_db}" WITH (FORCE)')


@pytest.mark.integration
@pytest.mark.parametrize(
    ("legacy_revision", "legacy_version_already_applied", "expected_new"),
    [
        (
            "0001_initial_schema",
            False,
            ["0002_add_upload_sessions_manifest"],
        ),
        (
            "ce06736ff293",
            True,
            [],
        ),
    ],
)
def test_upgrade_recognises_existing_alembic_revision(
    monkeypatch: pytest.MonkeyPatch,
    legacy_revision: str,
    legacy_version_already_applied: bool,
    expected_new: list[str],
) -> None:
    admin_url = TEST_DATABASE_URL.rsplit("/", 1)[0] + "/postgres"
    target_db = "eeg-lab_legacy_migration"
    target_url = TEST_DATABASE_URL.rsplit("/", 1)[0] + f"/{target_db}"

    with psycopg.connect(admin_url, autocommit=True) as admin:
        admin.execute(f'DROP DATABASE IF EXISTS "{target_db}" WITH (FORCE)')
        admin.execute(f'CREATE DATABASE "{target_db}"')

    monkeypatch.setenv("DATABASE_URL", target_url)
    clear_settings_cache()
    try:
        with psycopg.connect(target_url) as connection:
            connection.execute("CREATE TABLE alembic_version (version_num VARCHAR(32) NOT NULL)")
            connection.execute(
                "INSERT INTO alembic_version (version_num) VALUES (%s)",
                (legacy_revision,),
            )
            initial_schema = (
                Path(__file__).resolve().parents[2]
                / "migrations/versions/0001_initial_schema.up.sql"
            )
            connection.execute(initial_schema.read_text(encoding="utf-8"), prepare=False)
            if legacy_version_already_applied:
                upload_manifest = (
                    Path(__file__).resolve().parents[2]
                    / "migrations/versions/0002_add_upload_sessions_manifest.up.sql"
                )
                connection.execute(
                    upload_manifest.read_text(encoding="utf-8"),
                    prepare=False,
                )

        assert upgrade(target_url) == expected_new
        with psycopg.connect(target_url) as connection:
            assert connection.execute(
                """
                SELECT 1 FROM information_schema.columns
                WHERE table_schema = 'public'
                  AND table_name = 'upload_sessions'
                  AND column_name = 'manifest'
                """
            ).fetchone()
            applied = {
                row[0] for row in connection.execute("SELECT version FROM schema_migrations")
            }
        assert applied == {
            "0001_initial_schema",
            "0002_add_upload_sessions_manifest",
        }
    finally:
        clear_settings_cache()
        with psycopg.connect(admin_url, autocommit=True) as admin:
            admin.execute(f'DROP DATABASE IF EXISTS "{target_db}" WITH (FORCE)')
