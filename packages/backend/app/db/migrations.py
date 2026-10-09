"""Apply versioned SQL migration files to PostgreSQL."""

from __future__ import annotations

import argparse
import hashlib
import os
import re
from pathlib import Path
from typing import Any

import psycopg

MIGRATIONS_DIR = Path(__file__).resolve().parents[2] / "migrations" / "versions"
LEGACY_ALEMBIC_REVISIONS = {
    "0001_initial_schema": "0001_initial_schema",
    "ce06736ff293": "0002_add_upload_sessions_manifest",
}


def _migration_files(suffix: str) -> list[Path]:
    files = sorted(MIGRATIONS_DIR.glob(f"*.{suffix}.sql"))
    pattern = re.compile(r"^(\d{4})_[a-z0-9]+(?:_[a-z0-9]+)*$")
    sequences = []
    for path in files:
        match = pattern.fullmatch(path.stem.removesuffix(f".{suffix}"))
        if match is None:
            raise RuntimeError(
                f"Migration filename {path.name!r} must use "
                "'NNNN_descriptive_snake_case.{up,down}.sql'."
            )
        sequences.append(int(match.group(1)))
    if sequences != list(range(1, len(sequences) + 1)):
        raise RuntimeError(
            "SQL migration sequence numbers must be unique and contiguous from 0001."
        )
    return files


def _checksum(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _ensure_version_table(connection: psycopg.Connection[tuple[Any, ...]]) -> None:
    connection.execute(
        """
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            checksum TEXT NOT NULL,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )
        """
    )
    legacy_table = connection.execute("SELECT to_regclass('public.alembic_version')").fetchone()
    if legacy_table is None or legacy_table[0] is None:
        return

    legacy = connection.execute("SELECT version_num FROM alembic_version").fetchall()
    if not legacy:
        return
    up_files = _migration_files("up")
    version_names = [path.stem.removesuffix(".up") for path in up_files]
    for (revision,) in legacy:
        latest_version = LEGACY_ALEMBIC_REVISIONS.get(revision)
        if latest_version is None:
            raise RuntimeError(
                f"Cannot convert unknown Alembic revision {revision!r} to SQL migration history."
            )
        try:
            latest_index = version_names.index(latest_version)
        except ValueError as error:
            raise RuntimeError(
                f"Legacy migration {latest_version!r} has no SQL upgrade file."
            ) from error
        for path in up_files[: latest_index + 1]:
            migration_name = path.stem.removesuffix(".up")
            connection.execute(
                """
                INSERT INTO schema_migrations (version, checksum)
                VALUES (%s, %s)
                ON CONFLICT (version) DO NOTHING
                """,
                (migration_name, _checksum(path)),
            )


def upgrade(database_url: str) -> list[str]:
    """Apply all pending ``*.up.sql`` files in lexical order."""
    applied: list[str] = []
    with psycopg.connect(database_url) as connection:
        with connection.transaction():
            _ensure_version_table(connection)
        files = _migration_files("up")
        down_versions = {path.stem.removesuffix(".down") for path in _migration_files("down")}
        known_versions = {path.stem.removesuffix(".up") for path in files}
        if known_versions != down_versions:
            raise RuntimeError("Every SQL migration must have matching up and down files.")
        with connection.transaction():
            applied_versions = {
                row[0] for row in connection.execute("SELECT version FROM schema_migrations")
            }
        unknown_versions = applied_versions - known_versions
        if unknown_versions:
            raise RuntimeError(
                f"Database contains unknown migration versions: {sorted(unknown_versions)}"
            )

        for path in files:
            version = path.stem.removesuffix(".up")
            checksum = _checksum(path)
            with connection.transaction():
                row = connection.execute(
                    "SELECT checksum FROM schema_migrations WHERE version = %s",
                    (version,),
                ).fetchone()
                if row is not None:
                    if row[0] != checksum:
                        raise RuntimeError(
                            f"Applied migration {version!r} has changed; add a new migration."
                        )
                    continue

                down_path = MIGRATIONS_DIR / f"{version}.down.sql"
                if not down_path.is_file():
                    raise RuntimeError(f"Migration {version!r} is missing its down SQL file.")

                connection.execute(path.read_text(encoding="utf-8"), prepare=False)
                connection.execute(
                    "INSERT INTO schema_migrations (version, checksum) VALUES (%s, %s)",
                    (version, checksum),
                )
                applied.append(version)

    return applied


def downgrade(database_url: str) -> list[str]:
    """Revert all applied migrations in reverse lexical order."""
    reverted: list[str] = []
    with psycopg.connect(database_url) as connection:
        with connection.transaction():
            if (
                row := connection.execute(
                    "SELECT to_regclass('public.schema_migrations')"
                ).fetchone()
            ) is None or row[0] is None:
                return reverted

        rows = connection.execute("SELECT version FROM schema_migrations").fetchall()
        applied = {row[0] for row in rows}
        files = _migration_files("down")
        up_versions = {path.stem.removesuffix(".up") for path in _migration_files("up")}
        known_versions = {path.stem.removesuffix(".down") for path in files}
        if known_versions != up_versions:
            raise RuntimeError("Every SQL migration must have matching up and down files.")
        unknown = applied - known_versions
        if unknown:
            raise RuntimeError(f"Database contains unknown migration versions: {sorted(unknown)}")

        for path in reversed(files):
            version = path.stem.removesuffix(".down")
            if version not in applied:
                continue
            with connection.transaction():
                connection.execute(path.read_text(encoding="utf-8"), prepare=False)
                connection.execute(
                    "DELETE FROM schema_migrations WHERE version = %s",
                    (version,),
                )
            reverted.append(version)

        with connection.transaction():
            remaining = connection.execute("SELECT version FROM schema_migrations").fetchall()
            if not remaining:
                connection.execute("DROP TABLE schema_migrations")
                connection.execute("DROP TABLE IF EXISTS alembic_version")
    return reverted


def main() -> None:
    """CLI entry point for applying or reverting SQL migrations."""
    parser = argparse.ArgumentParser(description="Run EEG Lab SQL migrations.")
    parser.add_argument("direction", choices=("upgrade", "downgrade"))
    args = parser.parse_args()
    database_url = os.environ.get("DATABASE_URL")
    if database_url is None:
        from app.core.config import get_settings

        database_url = get_settings().database_url

    changed = upgrade(database_url) if args.direction == "upgrade" else downgrade(database_url)
    action = "Applied" if args.direction == "upgrade" else "Reverted"
    if changed:
        for version in changed:
            print(f"{action} {version}")
    else:
        print("Database schema is already up to date.")


if __name__ == "__main__":
    main()
