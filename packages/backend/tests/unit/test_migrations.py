"""Unit tests for migration file discovery and the CLI entry point."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest
from app.db import migrations

from tests.conftest import make_settings


def _write(directory: Path, name: str, content: str = "SELECT 1;") -> Path:
    path = directory / name
    path.write_text(content, encoding="utf-8")
    return path


def test_migration_files_rejects_malformed_filename(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(migrations, "MIGRATIONS_DIR", tmp_path)
    _write(tmp_path, "badname.up.sql")

    with pytest.raises(RuntimeError, match="must use"):
        migrations._migration_files("up")


def test_migration_files_requires_a_contiguous_sequence(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(migrations, "MIGRATIONS_DIR", tmp_path)
    _write(tmp_path, "0001_first.up.sql")
    _write(tmp_path, "0003_third.up.sql")

    with pytest.raises(RuntimeError, match="contiguous"):
        migrations._migration_files("up")


def test_migration_files_returns_lexical_order(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(migrations, "MIGRATIONS_DIR", tmp_path)
    _write(tmp_path, "0002_second.up.sql")
    _write(tmp_path, "0001_first.up.sql")

    files = migrations._migration_files("up")

    assert [path.name for path in files] == ["0001_first.up.sql", "0002_second.up.sql"]


def test_main_applies_pending_migrations(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setenv("DATABASE_URL", "postgresql://example")
    monkeypatch.setattr(migrations, "upgrade", lambda url: ["0001_initial_schema"])
    monkeypatch.setattr(sys, "argv", ["migrations", "upgrade"])

    migrations.main()

    assert "Applied 0001_initial_schema" in capsys.readouterr().out


def test_main_reports_when_already_up_to_date(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setenv("DATABASE_URL", "postgresql://example")
    monkeypatch.setattr(migrations, "downgrade", lambda url: [])
    monkeypatch.setattr(sys, "argv", ["migrations", "downgrade"])

    migrations.main()

    assert "already up to date" in capsys.readouterr().out


def test_main_falls_back_to_settings(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("DATABASE_URL", raising=False)
    settings = make_settings()
    monkeypatch.setattr("app.core.config.get_settings", lambda: settings)
    captured: dict[str, str] = {}

    def _record_url(database_url: str) -> list[str]:
        captured["url"] = database_url
        return ["0001_initial_schema"]

    monkeypatch.setattr(migrations, "upgrade", _record_url)
    monkeypatch.setattr(sys, "argv", ["migrations", "upgrade"])

    migrations.main()

    assert captured["url"] == settings.database_url
