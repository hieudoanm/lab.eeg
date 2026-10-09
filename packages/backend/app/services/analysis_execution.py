"""Analysis job execution and stale-job recovery.

The worker path is deliberately independent of the API process: it re-validates
its input, verifies the stored object against the dataset's recorded size and
checksum, computes deterministic statistics, and commits the result and the job
transition in one transaction. Because delivery may be duplicated or interrupted,
the executor claims the job atomically, tolerates duplicates, bounds retries, and
never reopens a terminal job.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Any
from uuid import UUID

import numpy as np
from pydantic import ValidationError

from app.core.config import Settings
from app.db.pool import Database
from app.models.analysis import (
    ANALYSIS_REQUESTED_EVENT,
    RESULT_SCHEMA_VERSION,
    AnalysisJob,
    AnalysisJobStatus,
    AnalysisType,
)
from app.models.dataset import Dataset, DatasetStatus
from app.repositories import analysis_jobs as jobs_repository
from app.repositories import analysis_results as results_repository
from app.repositories import audit
from app.repositories import datasets as datasets_repository
from app.repositories import outbox as outbox_repository
from app.schemas.analysis import (
    AnalysisProvenance,
    ChannelStatisticsRead,
    DescriptiveStatisticsParameters,
    DescriptiveStatisticsResult,
)
from app.scientific import (
    ChannelManifest,
    ChannelStatistics,
    RecordingInvalidError,
    StatisticsInvalidError,
    compute_descriptive_statistics,
    validate_recording,
)
from app.storage.base import Storage
from app.storage.recording import open_recording


class AnalysisExecutionError(Exception):
    """A classified execution failure.

    ``retryable`` distinguishes transient conditions (storage or database
    unavailability) from deterministic ones (invalid input), so retries stay
    bounded and never loop on a recording that can never succeed.
    """

    def __init__(self, code: str, message: str, *, retryable: bool) -> None:
        self.code = code
        self.message = message
        self.retryable = retryable
        super().__init__(message)


@dataclass(frozen=True, slots=True)
class ExecutionOutcome:
    """The result of attempting to execute one job."""

    job_id: UUID
    status: str
    result_id: UUID | None = None


@dataclass(frozen=True, slots=True)
class RecoveryReport:
    """Summary of a stale-job recovery run."""

    requeued: int
    failed: int
    batches: int


class _Rollback(Exception):
    """Internal signal to abandon a commit whose state guard failed."""


def _now() -> datetime:
    return datetime.now(UTC)


def _lease(settings: Settings) -> timedelta:
    """A lease that outlives the task time limit so live work is never reclaimed."""
    seconds = max(
        settings.job_lease_seconds,
        settings.celery_task_time_limit_seconds + 30,
    )
    return timedelta(seconds=seconds)


async def execute_analysis_job(
    database: Database,
    storage: Storage,
    settings: Settings,
    *,
    job_id: UUID,
) -> ExecutionOutcome:
    """Claim and execute a queued job, committing a result or a failure."""
    claimed = await _claim(database, settings, job_id)
    if claimed is None:
        return ExecutionOutcome(job_id=job_id, status="skipped")

    async with database.connection() as connection:
        dataset = await datasets_repository.get_dataset(
            connection,
            organisation_id=claimed.organisation_id,
            dataset_id=claimed.dataset_id,
        )

    try:
        results_payload, provenance_payload = await _compute(storage, settings, claimed, dataset)
    except AnalysisExecutionError as exc:
        return await _handle_failure(database, claimed, exc)
    except Exception:
        # Storage or database hiccups are transient; retry within bounded attempts.
        return await _handle_failure(
            database,
            claimed,
            AnalysisExecutionError(
                "analysis_execution_failed",
                "The analysis could not be completed.",
                retryable=True,
            ),
        )

    return await _commit(database, claimed, results_payload, provenance_payload)


async def _claim(
    database: Database,
    settings: Settings,
    job_id: UUID,
) -> AnalysisJob | None:
    async with database.transaction() as connection:
        job = await jobs_repository.get_job_by_id(connection, job_id=job_id)
        if job is None or job.status is not AnalysisJobStatus.QUEUED:
            return None
        return await jobs_repository.claim_job(
            connection,
            job_id=job_id,
            lease_expires_at=_now() + _lease(settings),
        )


async def _commit(
    database: Database,
    job: AnalysisJob,
    results_payload: dict[str, Any],
    provenance_payload: dict[str, Any],
) -> ExecutionOutcome:
    try:
        async with database.transaction() as connection:
            existing = await results_repository.get_result_by_job_id(connection, job_id=job.id)
            if existing is not None:
                await jobs_repository.complete_job(connection, job_id=job.id)
                return ExecutionOutcome(job_id=job.id, status="succeeded", result_id=existing.id)

            created = await results_repository.create_result(
                connection,
                organisation_id=job.organisation_id,
                job_id=job.id,
                dataset_id=job.dataset_id,
                dataset_checksum_sha256=provenance_payload.get("input_checksum_sha256"),
                algorithm_version=job.algorithm_version,
                result_schema_version=RESULT_SCHEMA_VERSION,
                results=results_payload,
                provenance=provenance_payload,
            )
            if created is None:
                # A concurrent duplicate committed first; reuse its result.
                raced = await results_repository.get_result_by_job_id(connection, job_id=job.id)
                if raced is None:
                    raise _Rollback
                created = raced

            completed = await jobs_repository.complete_job(connection, job_id=job.id)
            if completed is None:
                # The job is no longer running (cancelled or reclaimed): roll back.
                raise _Rollback
            await audit.record_audit_event(
                connection,
                action="analysis.completed",
                resource_type="analysis_job",
                organisation_id=job.organisation_id,
                resource_id=job.id,
                metadata={"dataset_id": str(job.dataset_id)},
            )
    except _Rollback:
        return ExecutionOutcome(job_id=job.id, status="skipped")
    return ExecutionOutcome(job_id=job.id, status="succeeded", result_id=created.id)


async def _handle_failure(
    database: Database,
    job: AnalysisJob,
    error: AnalysisExecutionError,
) -> ExecutionOutcome:
    async with database.transaction() as connection:
        if not error.retryable or job.attempt_count >= job.max_attempts:
            failed = await jobs_repository.fail_job(
                connection,
                job_id=job.id,
                error_code=error.code,
                error_message=error.message,
            )
            if failed is not None:
                await audit.record_audit_event(
                    connection,
                    action="analysis.failed",
                    resource_type="analysis_job",
                    organisation_id=job.organisation_id,
                    resource_id=job.id,
                    metadata={"dataset_id": str(job.dataset_id), "error_code": error.code},
                )
            return ExecutionOutcome(job_id=job.id, status="failed")

        requeued = await jobs_repository.requeue_job(connection, job_id=job.id)
        if requeued is None:
            return ExecutionOutcome(job_id=job.id, status="skipped")
        await outbox_repository.create_event(
            connection,
            organisation_id=job.organisation_id,
            event_type=ANALYSIS_REQUESTED_EVENT,
            aggregate_type="analysis_job",
            aggregate_id=job.id,
            payload={
                "job_id": str(job.id),
                "dataset_id": str(job.dataset_id),
                "analysis_type": job.analysis_type.value,
            },
        )
        return ExecutionOutcome(job_id=job.id, status="requeued")


async def _compute(
    storage: Storage,
    settings: Settings,
    job: AnalysisJob,
    dataset: Dataset | None,
) -> tuple[dict[str, Any], dict[str, Any]]:
    if job.analysis_type is not AnalysisType.DESCRIPTIVE_STATISTICS:
        raise AnalysisExecutionError(
            "analysis_type_unsupported",
            "This analysis type is not supported.",
            retryable=False,
        )
    if dataset is None or dataset.status is not DatasetStatus.READY:
        raise AnalysisExecutionError(
            "analysis_input_unavailable",
            "The dataset is not available for analysis.",
            retryable=False,
        )
    key = dataset.storage_key
    if key is None:
        raise AnalysisExecutionError(
            "analysis_input_unavailable",
            "The dataset is not available for analysis.",
            retryable=False,
        )

    if not await storage.exists(key):
        raise AnalysisExecutionError(
            "analysis_input_unavailable",
            "The recording could not be found.",
            retryable=True,
        )
    size = await storage.size(key)
    if dataset.file_size_bytes is not None and size != dataset.file_size_bytes:
        raise AnalysisExecutionError(
            "analysis_input_changed",
            "The recording has changed since it was validated.",
            retryable=False,
        )
    if dataset.checksum_sha256 is not None:
        checksum = await storage.hash_sha256(key)
        if checksum != dataset.checksum_sha256:
            raise AnalysisExecutionError(
                "analysis_input_changed",
                "The recording has changed since it was validated.",
                retryable=False,
            )

    try:
        parameters = DescriptiveStatisticsParameters.model_validate(job.parameters)
    except ValidationError as exc:
        raise AnalysisExecutionError(
            "analysis_parameters_invalid",
            "The analysis parameters are invalid.",
            retryable=False,
        ) from exc

    manifest = ChannelManifest(
        sampling_frequency_hz=dataset.sampling_frequency_hz,
        channel_names=dataset.channel_names or [],
        channel_units=dataset.channel_units,
    )

    async with open_recording(storage, key, max_bytes=settings.max_upload_bytes) as path:
        try:
            metadata = validate_recording(
                path,
                manifest=manifest,
                max_channels=settings.max_channels,
                max_sample_count=settings.max_sample_count,
            )
            values = _load_values(path)
            channel_names = dataset.channel_names or [
                f"channel_{index}" for index in range(metadata.channel_count)
            ]
            statistics = compute_descriptive_statistics(
                values,
                channel_names=channel_names,
                channel_units=dataset.channel_units,
                include_mean=parameters.include_mean,
                include_standard_deviation=parameters.include_standard_deviation,
                include_min_max=parameters.include_min_max,
                include_peak_to_peak=parameters.include_peak_to_peak,
            )
        except RecordingInvalidError as exc:
            raise AnalysisExecutionError(
                "analysis_input_invalid",
                str(exc),
                retryable=False,
            ) from exc
        except StatisticsInvalidError as exc:
            raise AnalysisExecutionError(
                "analysis_input_invalid",
                str(exc),
                retryable=False,
            ) from exc

    result = DescriptiveStatisticsResult(
        channels=[_channel_payload(statistic) for statistic in statistics]
    )
    provenance = AnalysisProvenance(
        input_checksum_sha256=dataset.checksum_sha256,
        sampling_frequency_hz=dataset.sampling_frequency_hz,
        channel_count=metadata.channel_count,
        sample_count=metadata.sample_count,
        parameters=parameters,
    )
    return result.model_dump(mode="json"), provenance.model_dump(mode="json")


def _channel_payload(statistic: ChannelStatistics) -> ChannelStatisticsRead:
    return ChannelStatisticsRead(
        channel_index=statistic.channel_index,
        channel_name=statistic.channel_name,
        unit=statistic.unit,
        mean=statistic.mean,
        standard_deviation=statistic.standard_deviation,
        minimum=statistic.minimum,
        maximum=statistic.maximum,
        peak_to_peak=statistic.peak_to_peak,
        valid_sample_count=statistic.valid_sample_count,
    )


def _load_values(path: Path) -> np.ndarray[Any, np.dtype[Any]]:
    memmap = np.load(path, mmap_mode="r", allow_pickle=False)
    try:
        return np.asarray(memmap, dtype=np.float64)
    finally:
        close = getattr(memmap, "close", None)
        if callable(close):
            close()


async def recover_stale_jobs(
    database: Database,
    settings: Settings,
    *,
    now: datetime | None = None,
    batch_size: int = 100,
    max_batches: int = 20,
) -> RecoveryReport:
    """Requeue or fail running jobs whose worker lease has expired.

    A stale job with attempts remaining is requeued with a fresh outbox event so
    it is redelivered; one that has exhausted its attempts is failed with a safe,
    visible error code.
    """
    if batch_size < 1:
        raise ValueError("batch_size must be at least 1")
    if max_batches < 1:
        raise ValueError("max_batches must be at least 1")

    deadline = now or _now()
    requeued = failed = batches = 0
    while batches < max_batches:
        async with database.transaction() as connection:
            stale = await jobs_repository.claim_stale_jobs(
                connection,
                now=deadline,
                limit=batch_size,
            )
            if not stale:
                break
            for job in stale:
                if job.attempt_count >= job.max_attempts:
                    transitioned = await jobs_repository.fail_job(
                        connection,
                        job_id=job.id,
                        error_code="analysis_stale",
                        error_message="The analysis did not complete within its lease.",
                    )
                    if transitioned is not None:
                        failed += 1
                    continue
                transitioned = await jobs_repository.requeue_job(connection, job_id=job.id)
                if transitioned is None:
                    continue
                await outbox_repository.create_event(
                    connection,
                    organisation_id=job.organisation_id,
                    event_type=ANALYSIS_REQUESTED_EVENT,
                    aggregate_type="analysis_job",
                    aggregate_id=job.id,
                    payload={
                        "job_id": str(job.id),
                        "dataset_id": str(job.dataset_id),
                        "analysis_type": job.analysis_type.value,
                    },
                )
                requeued += 1
        batches += 1
        if len(stale) < batch_size:
            break

    return RecoveryReport(requeued=requeued, failed=failed, batches=batches)


__all__ = [
    "AnalysisExecutionError",
    "ExecutionOutcome",
    "RecoveryReport",
    "execute_analysis_job",
    "recover_stale_jobs",
]
