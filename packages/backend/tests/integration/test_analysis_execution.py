"""Integration tests for analysis execution, recovery, and outbox dispatch."""

from __future__ import annotations

import hashlib
import io
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Any
from uuid import UUID, uuid4

import numpy as np
import psycopg
import pytest
from app.db.pool import Database
from app.models.analysis import AnalysisJob, AnalysisType
from app.models.dataset import Dataset, DatasetFormat
from app.repositories import analysis_jobs as jobs_repository
from app.repositories import datasets as datasets_repository
from app.repositories import outbox as outbox_repository
from app.services.analysis_execution import execute_analysis_job, recover_stale_jobs
from app.services.outbox import dispatch_pending_events
from app.storage.local import LocalStorage

from tests.conftest import (
    TEST_STORAGE_DIR,
    create_membership,
    create_organisation,
    create_user,
    make_settings,
)

pytestmark = pytest.mark.integration

_PARAMETERS = {
    "include_mean": True,
    "include_standard_deviation": True,
    "include_min_max": True,
    "include_peak_to_peak": True,
}


def _npy_bytes(array: np.ndarray[Any, np.dtype[Any]]) -> bytes:
    buffer = io.BytesIO()
    np.save(buffer, array, allow_pickle=False)
    return buffer.getvalue()


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


async def _seed_ready_dataset(
    database: Database,
    storage: LocalStorage,
    *,
    organisation_id: UUID,
    study_id: UUID,
    uploaded_by: UUID,
    array: np.ndarray[Any, np.dtype[Any]],
    object_bytes: bytes | None = None,
    channel_names: list[str] | None = None,
    channel_units: list[str] | None = None,
) -> tuple[Dataset, str]:
    key = f"organisations/{organisation_id}/datasets/{uuid4()}/recording.npy"
    payload = object_bytes if object_bytes is not None else _npy_bytes(array)
    await storage.write_bytes(key, payload)
    checksum = hashlib.sha256(payload).hexdigest()
    names = channel_names or [f"C{index}" for index in range(int(array.shape[0]))]
    async with database.transaction() as connection:
        created = await datasets_repository.create_dataset(
            connection,
            organisation_id=organisation_id,
            study_id=study_id,
            name="Recording",
            description=None,
            dataset_format=DatasetFormat.NPY,
            source_description=None,
            metadata={},
            uploaded_by=uploaded_by,
        )
        updated = await datasets_repository.mark_dataset_validated(
            connection,
            organisation_id=organisation_id,
            dataset_id=created.id,
            storage_key=key,
            checksum_sha256=checksum,
            file_size_bytes=len(payload),
            sampling_frequency_hz=256.0,
            channel_count=int(array.shape[0]),
            sample_count=int(array.shape[1]),
            duration_seconds=float(array.shape[1]) / 256.0,
            channel_names=names,
            channel_units=channel_units,
            metadata={},
        )
    assert updated is not None
    return updated, key


async def _create_job(
    database: Database,
    dataset: Dataset,
    *,
    max_attempts: int = 3,
    parameters: dict[str, Any] | None = None,
) -> AnalysisJob:
    async with database.transaction() as connection:
        return await jobs_repository.create_job(
            connection,
            organisation_id=dataset.organisation_id,
            dataset_id=dataset.id,
            requested_by=dataset.uploaded_by,
            analysis_type=AnalysisType.DESCRIPTIVE_STATISTICS,
            algorithm_version="1.0.0",
            parameters=parameters or dict(_PARAMETERS),
            idempotency_key=None,
            max_attempts=max_attempts,
        )


async def _seed(
    database: Database,
    storage: LocalStorage,
    db: psycopg.Connection[dict[str, Any]],
    *,
    array: np.ndarray[Any, np.dtype[Any]] | None = None,
    object_bytes: bytes | None = None,
) -> tuple[Dataset, str]:
    user_id = create_user(db, subject="analyst")
    organisation_id = create_organisation(db, name="Analysis Org")
    create_membership(db, organisation_id=organisation_id, user_id=user_id, role="owner")
    study_id = await _create_study(database, organisation_id, user_id)
    default = array if array is not None else np.arange(8, dtype=np.float64).reshape(2, 4)
    return await _seed_ready_dataset(
        database,
        storage,
        organisation_id=organisation_id,
        study_id=study_id,
        uploaded_by=user_id,
        array=default,
        object_bytes=object_bytes,
        channel_units=["uV", "uV"],
    )


def _job_status(db: psycopg.Connection[dict[str, Any]], job_id: UUID) -> str:
    row = db.execute("SELECT status FROM analysis_jobs WHERE id = %s", (job_id,)).fetchone()
    assert row is not None
    return str(row["status"])


def _count(db: psycopg.Connection[dict[str, Any]], table: str) -> int:
    row = db.execute(f"SELECT COUNT(*) AS count FROM {table}").fetchone()
    assert row is not None
    return int(row["count"])


async def test_execute_success_persists_result_and_completes(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    storage = LocalStorage(Path(TEST_STORAGE_DIR))
    settings = make_settings()
    dataset, _ = await _seed(database, storage, db)
    job = await _create_job(database, dataset)

    outcome = await execute_analysis_job(database, storage, settings, job_id=job.id)

    assert outcome.status == "succeeded"
    assert outcome.result_id is not None
    assert _job_status(db, job.id) == "succeeded"
    row = db.execute(
        "SELECT results, provenance, dataset_checksum_sha256 FROM analysis_results WHERE job_id = %s",
        (job.id,),
    ).fetchone()
    assert row is not None
    assert row["dataset_checksum_sha256"] == dataset.checksum_sha256
    assert row["provenance"]["input_checksum_sha256"] == dataset.checksum_sha256
    assert len(row["results"]["channels"]) == 2
    assert row["results"]["channels"][0]["channel_name"] == "C0"
    assert row["results"]["channels"][0]["valid_sample_count"] == 4
    completed = db.execute(
        "SELECT COUNT(*) AS count FROM audit_events WHERE action = 'analysis.completed'"
    ).fetchone()
    assert completed is not None and completed["count"] == 1


async def test_duplicate_delivery_commits_one_result(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    storage = LocalStorage(Path(TEST_STORAGE_DIR))
    settings = make_settings()
    dataset, _ = await _seed(database, storage, db)
    job = await _create_job(database, dataset)

    first = await execute_analysis_job(database, storage, settings, job_id=job.id)
    second = await execute_analysis_job(database, storage, settings, job_id=job.id)

    assert first.status == "succeeded"
    assert second.status == "skipped"
    assert _count(db, "analysis_results") == 1
    assert _job_status(db, job.id) == "succeeded"


async def test_checksum_mismatch_fails_without_committing_a_result(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    storage = LocalStorage(Path(TEST_STORAGE_DIR))
    settings = make_settings()
    dataset, key = await _seed(database, storage, db)
    job = await _create_job(database, dataset)
    await storage.write_bytes(key, _npy_bytes(np.ones((2, 4), dtype=np.float64)))

    outcome = await execute_analysis_job(database, storage, settings, job_id=job.id)

    assert outcome.status == "failed"
    assert _job_status(db, job.id) == "failed"
    row = db.execute("SELECT error_code FROM analysis_jobs WHERE id = %s", (job.id,)).fetchone()
    assert row is not None and row["error_code"] == "analysis_input_changed"
    assert _count(db, "analysis_results") == 0


async def test_invalid_recording_fails_as_non_retryable(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    storage = LocalStorage(Path(TEST_STORAGE_DIR))
    settings = make_settings()
    dataset, _ = await _seed(database, storage, db, object_bytes=b"not a npy file")
    job = await _create_job(database, dataset, max_attempts=3)

    outcome = await execute_analysis_job(database, storage, settings, job_id=job.id)

    assert outcome.status == "failed"
    row = db.execute("SELECT error_code FROM analysis_jobs WHERE id = %s", (job.id,)).fetchone()
    assert row is not None and row["error_code"] == "analysis_input_invalid"
    assert _count(db, "analysis_results") == 0


class _FlakyStorage(LocalStorage):
    """Storage double whose ``size`` fails until the flag is cleared."""

    def __init__(self, root: Path) -> None:
        super().__init__(root)
        self.fail = True

    async def size(self, key: str) -> int:
        if self.fail:
            raise RuntimeError("storage unavailable")
        return await super().size(key)


async def test_transient_failure_requeues_then_succeeds(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    settings = make_settings()
    storage = _FlakyStorage(Path(TEST_STORAGE_DIR))
    dataset, _ = await _seed(database, storage, db)
    job = await _create_job(database, dataset, max_attempts=2)

    first = await execute_analysis_job(database, storage, settings, job_id=job.id)
    assert first.status == "requeued"
    assert _job_status(db, job.id) == "queued"
    assert _count(db, "outbox_events") == 1

    storage.fail = False
    second = await execute_analysis_job(database, storage, settings, job_id=job.id)

    assert second.status == "succeeded"
    assert _job_status(db, job.id) == "succeeded"
    assert _count(db, "analysis_results") == 1


async def test_exhausted_retries_fail_the_job(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    settings = make_settings()
    storage = _FlakyStorage(Path(TEST_STORAGE_DIR))
    dataset, _ = await _seed(database, storage, db)
    job = await _create_job(database, dataset, max_attempts=1)

    outcome = await execute_analysis_job(database, storage, settings, job_id=job.id)

    assert outcome.status == "failed"
    row = db.execute("SELECT error_code FROM analysis_jobs WHERE id = %s", (job.id,)).fetchone()
    assert row is not None and row["error_code"] == "analysis_execution_failed"


async def test_recover_stale_jobs_requeues_then_fails(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    settings = make_settings()
    storage = LocalStorage(Path(TEST_STORAGE_DIR))
    dataset, _ = await _seed(database, storage, db)
    recoverable = await _create_job(database, dataset, max_attempts=3)
    exhausted = await _create_job(database, dataset, max_attempts=2)

    db.execute(
        """
        UPDATE analysis_jobs
        SET status = 'running', attempt_count = 1,
            lease_expires_at = now() - interval '1 hour'
        WHERE id = %s
        """,
        (recoverable.id,),
    )
    db.execute(
        """
        UPDATE analysis_jobs
        SET status = 'running', attempt_count = 2,
            lease_expires_at = now() - interval '1 hour'
        WHERE id = %s
        """,
        (exhausted.id,),
    )

    report = await recover_stale_jobs(database, settings)

    assert report.requeued == 1
    assert report.failed == 1
    assert _job_status(db, recoverable.id) == "queued"
    assert _job_status(db, exhausted.id) == "failed"
    assert _count(db, "outbox_events") == 1


async def test_job_and_outbox_event_roll_back_together(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    storage = LocalStorage(Path(TEST_STORAGE_DIR))
    dataset, _ = await _seed(database, storage, db)

    class _Abort(Exception):
        pass

    with pytest.raises(_Abort):
        async with database.transaction() as connection:
            job = await jobs_repository.create_job(
                connection,
                organisation_id=dataset.organisation_id,
                dataset_id=dataset.id,
                requested_by=dataset.uploaded_by,
                analysis_type=AnalysisType.DESCRIPTIVE_STATISTICS,
                algorithm_version="1.0.0",
                parameters=dict(_PARAMETERS),
                idempotency_key=None,
                max_attempts=3,
            )
            await outbox_repository.create_event(
                connection,
                organisation_id=dataset.organisation_id,
                event_type="analysis.requested",
                aggregate_type="analysis_job",
                aggregate_id=job.id,
                payload={"job_id": str(job.id)},
            )
            raise _Abort

    assert _count(db, "analysis_jobs") == 0
    assert _count(db, "outbox_events") == 0


class _RecordingPublisher:
    """Publisher double that records published tasks or fails on demand."""

    def __init__(self, *, fail: bool = False) -> None:
        self.fail = fail
        self.calls: list[tuple[str, dict[str, Any]]] = []

    def publish(self, *, task_name: str, payload: dict[str, Any]) -> None:
        if self.fail:
            raise RuntimeError("broker unavailable")
        self.calls.append((task_name, payload))


async def _create_event(database: Database, dataset: Dataset) -> UUID:
    async with database.transaction() as connection:
        event = await outbox_repository.create_event(
            connection,
            organisation_id=dataset.organisation_id,
            event_type="analysis.requested",
            aggregate_type="analysis_job",
            aggregate_id=dataset.id,
            payload={"job_id": str(uuid4())},
        )
    return event.id


async def test_dispatch_publishes_events_once(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    storage = LocalStorage(Path(TEST_STORAGE_DIR))
    dataset, _ = await _seed(database, storage, db)
    await _create_event(database, dataset)

    publisher = _RecordingPublisher()
    report = await dispatch_pending_events(database, publisher)

    assert report.published == 1
    assert len(publisher.calls) == 1
    assert publisher.calls[0][0] == "eeg-lab.run_analysis"
    assert _count(db, "outbox_events WHERE published_at IS NOT NULL") == 1

    again = await dispatch_pending_events(database, publisher)
    assert again.claimed == 0


async def test_dispatch_retries_then_succeeds(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    storage = LocalStorage(Path(TEST_STORAGE_DIR))
    dataset, _ = await _seed(database, storage, db)
    await _create_event(database, dataset)

    failing = _RecordingPublisher(fail=True)
    first = await dispatch_pending_events(database, failing, max_attempts=3, retry_delay_seconds=0)
    assert first.failed == 1
    assert first.published == 0
    assert _count(db, "outbox_events WHERE published_at IS NULL") == 1

    good = _RecordingPublisher()
    second = await dispatch_pending_events(
        database,
        good,
        now=datetime.now(UTC) + timedelta(seconds=1),
        max_attempts=3,
        retry_delay_seconds=0,
    )
    assert second.published == 1
    assert len(good.calls) == 1


async def test_dispatch_parks_exhausted_events(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    storage = LocalStorage(Path(TEST_STORAGE_DIR))
    dataset, _ = await _seed(database, storage, db)
    await _create_event(database, dataset)

    report = await dispatch_pending_events(
        database, _RecordingPublisher(fail=True), max_attempts=1, retry_delay_seconds=0
    )

    assert report.exhausted == 1
    assert report.published == 0
    row = db.execute("SELECT last_error_code, next_attempt_at FROM outbox_events").fetchone()
    assert row is not None
    assert row["last_error_code"] == "outbox_dispatch_exhausted"
    assert row["next_attempt_at"] > datetime.now(UTC) + timedelta(days=1)


class _RemoteStorage(LocalStorage):
    """Storage double without a local path, forcing object materialisation."""

    def local_path(self, key: str) -> Path | None:
        return None


async def test_execute_materialises_a_remote_recording(
    database: Database, db: psycopg.Connection[dict[str, Any]]
) -> None:
    storage = _RemoteStorage(Path(TEST_STORAGE_DIR))
    settings = make_settings()
    dataset, _ = await _seed(database, storage, db)
    job = await _create_job(database, dataset)

    outcome = await execute_analysis_job(database, storage, settings, job_id=job.id)

    assert outcome.status == "succeeded"
    assert _job_status(db, job.id) == "succeeded"


async def test_recover_stale_jobs_validates_batch_size(database: Database) -> None:
    with pytest.raises(ValueError):
        await recover_stale_jobs(database, make_settings(), batch_size=0)


async def test_recover_stale_jobs_validates_max_batches(database: Database) -> None:
    with pytest.raises(ValueError):
        await recover_stale_jobs(database, make_settings(), max_batches=0)
