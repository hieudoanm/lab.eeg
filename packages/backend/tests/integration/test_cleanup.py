"""Integration tests for abandoned-upload cleanup (service + real database)."""

from __future__ import annotations

from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Any
from uuid import UUID

import psycopg
import pytest
from app.db.pool import Database
from app.models.dataset import DatasetFormat
from app.repositories import upload_sessions as sessions_repository
from app.services.uploads import cleanup_abandoned_uploads
from app.storage.local import LocalStorage

from tests.conftest import (
    TEST_STORAGE_DIR,
    create_membership,
    create_organisation,
    create_user,
)

pytestmark = pytest.mark.integration


async def _create_study(database: Database, organisation_id: UUID, created_by: UUID) -> UUID:
    async with database.connection() as connection:
        cursor = await connection.execute(
            """
            INSERT INTO studies (organisation_id, name, description, created_by)
            VALUES (%s, 'Study', NULL, %s)
            RETURNING id
            """,
            (organisation_id, created_by),
        )
        row = await cursor.fetchone()
    assert row is not None
    return UUID(str(row["id"]))


async def _create_dataset(
    database: Database, organisation_id: UUID, study_id: UUID, uploaded_by: UUID
) -> UUID:
    async with database.connection() as connection:
        cursor = await connection.execute(
            """
            INSERT INTO datasets (
                organisation_id, study_id, name, description, format,
                source_description, metadata, uploaded_by
            )
            VALUES (%s, %s, 'Recording', NULL, %s, NULL, '{}'::jsonb, %s)
            RETURNING id
            """,
            (organisation_id, study_id, DatasetFormat.NPY.value, uploaded_by),
        )
        row = await cursor.fetchone()
    assert row is not None
    return UUID(str(row["id"]))


async def _create_session(
    database: Database,
    *,
    organisation_id: UUID,
    dataset_id: UUID,
    created_by: UUID,
    object_key: str,
    expires_at: datetime,
    status: str = "created",
) -> UUID:
    async with database.transaction() as connection:
        session = await sessions_repository.create_upload_session(
            connection,
            organisation_id=organisation_id,
            dataset_id=dataset_id,
            created_by=created_by,
            object_key=object_key,
            expected_size_bytes=None,
            expected_checksum_sha256=None,
            expires_at=expires_at,
            manifest={},
        )
        if status != "created":
            await connection.execute(
                "UPDATE upload_sessions SET status = %s WHERE id = %s",
                (status, session.id),
            )
    return session.id


def _session_status(db: psycopg.Connection[dict[str, Any]], session_id: UUID) -> str:
    row = db.execute("SELECT status FROM upload_sessions WHERE id = %s", (session_id,)).fetchone()
    assert row is not None
    return str(row["status"])


async def _seed(
    db: psycopg.Connection[dict[str, Any]], database: Database
) -> tuple[UUID, UUID, UUID]:
    user_id = create_user(db, subject="cleaner")
    organisation_id = create_organisation(db, name="Cleanup Org")
    create_membership(db, organisation_id=organisation_id, user_id=user_id, role="owner")
    study_id = await _create_study(database, organisation_id, user_id)
    dataset_id = await _create_dataset(database, organisation_id, study_id, user_id)
    return user_id, organisation_id, dataset_id


async def test_cleanup_expires_sessions_and_deletes_objects(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    user_id, organisation_id, dataset_id = await _seed(db, database)
    storage = LocalStorage(Path(TEST_STORAGE_DIR))
    past = datetime.now(UTC) - timedelta(hours=1)

    first_key = f"organisations/{organisation_id}/datasets/{dataset_id}/first.npy"
    second_key = f"organisations/{organisation_id}/datasets/{dataset_id}/second.npy"
    await storage.write_bytes(first_key, b"first")
    await storage.write_bytes(second_key, b"second")

    first = await _create_session(
        database,
        organisation_id=organisation_id,
        dataset_id=dataset_id,
        created_by=user_id,
        object_key=first_key,
        expires_at=past,
        status="uploaded",
    )
    second = await _create_session(
        database,
        organisation_id=organisation_id,
        dataset_id=dataset_id,
        created_by=user_id,
        object_key=second_key,
        expires_at=past,
    )

    report = await cleanup_abandoned_uploads(database, storage)

    assert report.expired_sessions == 2
    assert report.objects_deleted == 2
    assert report.batches == 1
    assert _session_status(db, first) == "expired"
    assert _session_status(db, second) == "expired"
    assert await storage.exists(first_key) is False
    assert await storage.exists(second_key) is False


async def test_cleanup_ignores_future_and_terminal_sessions(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    user_id, organisation_id, dataset_id = await _seed(db, database)
    storage = LocalStorage(Path(TEST_STORAGE_DIR))
    future = datetime.now(UTC) + timedelta(hours=1)

    live_key = f"organisations/{organisation_id}/datasets/{dataset_id}/live.npy"
    done_key = f"organisations/{organisation_id}/datasets/{dataset_id}/done.npy"
    await storage.write_bytes(live_key, b"live")
    await storage.write_bytes(done_key, b"done")

    live = await _create_session(
        database,
        organisation_id=organisation_id,
        dataset_id=dataset_id,
        created_by=user_id,
        object_key=live_key,
        expires_at=future,
    )
    done = await _create_session(
        database,
        organisation_id=organisation_id,
        dataset_id=dataset_id,
        created_by=user_id,
        object_key=done_key,
        expires_at=future,
        status="completed",
    )

    report = await cleanup_abandoned_uploads(database, storage)

    assert report.expired_sessions == 0
    assert report.objects_deleted == 0
    assert report.batches == 0
    assert _session_status(db, live) == "created"
    assert _session_status(db, done) == "completed"
    assert await storage.exists(live_key) is True
    assert await storage.exists(done_key) is True


async def test_cleanup_tolerates_a_missing_object(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    user_id, organisation_id, dataset_id = await _seed(db, database)
    storage = LocalStorage(Path(TEST_STORAGE_DIR))
    missing_key = f"organisations/{organisation_id}/datasets/{dataset_id}/missing.npy"

    session_id = await _create_session(
        database,
        organisation_id=organisation_id,
        dataset_id=dataset_id,
        created_by=user_id,
        object_key=missing_key,
        expires_at=datetime.now(UTC) - timedelta(minutes=5),
    )

    report = await cleanup_abandoned_uploads(database, storage)

    assert report.expired_sessions == 1
    assert report.objects_deleted == 0
    assert _session_status(db, session_id) == "expired"


async def test_cleanup_processes_bounded_batches(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    user_id, organisation_id, dataset_id = await _seed(db, database)
    storage = LocalStorage(Path(TEST_STORAGE_DIR))
    past = datetime.now(UTC) - timedelta(hours=1)

    for index in range(3):
        await _create_session(
            database,
            organisation_id=organisation_id,
            dataset_id=dataset_id,
            created_by=user_id,
            object_key=f"organisations/{organisation_id}/datasets/{dataset_id}/{index}.npy",
            expires_at=past,
        )

    report = await cleanup_abandoned_uploads(database, storage, batch_size=1, max_batches=10)

    assert report.batches == 3
    assert report.expired_sessions == 3

    for index in range(2):
        await _create_session(
            database,
            organisation_id=organisation_id,
            dataset_id=dataset_id,
            created_by=user_id,
            object_key=f"organisations/{organisation_id}/datasets/{dataset_id}/b{index}.npy",
            expires_at=past,
        )

    capped = await cleanup_abandoned_uploads(database, storage, batch_size=1, max_batches=1)
    assert capped.batches == 1
    assert capped.expired_sessions == 1


class _FailingDeleteStorage(LocalStorage):
    """Storage double whose deletions always fail."""

    async def delete(self, key: str) -> None:
        raise RuntimeError("storage unavailable")


async def test_cleanup_records_deletion_failures_without_aborting(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    user_id, organisation_id, dataset_id = await _seed(db, database)
    storage = LocalStorage(Path(TEST_STORAGE_DIR))
    key = f"organisations/{organisation_id}/datasets/{dataset_id}/stuck.npy"
    await storage.write_bytes(key, b"payload")

    session_id = await _create_session(
        database,
        organisation_id=organisation_id,
        dataset_id=dataset_id,
        created_by=user_id,
        object_key=key,
        expires_at=datetime.now(UTC) - timedelta(minutes=1),
    )

    report = await cleanup_abandoned_uploads(
        database, _FailingDeleteStorage(Path(TEST_STORAGE_DIR))
    )

    assert report.expired_sessions == 1
    assert report.objects_deleted == 0
    assert report.objects_failed == 1
    assert _session_status(db, session_id) == "expired"
    assert await storage.exists(key) is True
