"""Analysis operations: submission, idempotency, status, results, and cancellation.

Submission is the transactional core of the pipeline: within a single database
transaction the service verifies the dataset, reserves the idempotency key,
creates the job, and appends its outbox event. A committed job therefore always
has a durable dispatch event, and a rolled-back submission leaves neither.
"""

from __future__ import annotations

import hashlib
import json
from datetime import UTC, datetime, timedelta
from typing import Any
from uuid import UUID

from psycopg import AsyncConnection

from app.core.config import Settings
from app.core.exceptions import ConflictError, NotFoundError
from app.core.pagination import decode_cursor, encode_cursor
from app.db.pool import Database
from app.models.analysis import (
    ANALYSIS_REQUESTED_EVENT,
    AnalysisJob,
    AnalysisJobStatus,
    AnalysisResult,
)
from app.models.dataset import DatasetStatus
from app.models.identity import OrganisationContext, Role
from app.repositories import analysis_jobs as jobs_repository
from app.repositories import analysis_results as results_repository
from app.repositories import audit
from app.repositories import datasets as datasets_repository
from app.repositories import idempotency as idempotency_repository
from app.repositories import outbox as outbox_repository
from app.schemas.analysis import AnalysisSubmitRequest
from app.schemas.common import format_utc
from app.services import tenancy

_WRITE_ROLES = frozenset({Role.OWNER, Role.RESEARCHER})

#: Idempotency operation identifier for analysis submission.
SUBMIT_OPERATION = "analysis.submit"


def _now() -> datetime:
    return datetime.now(UTC)


def _request_hash(*, dataset_id: UUID, payload: AnalysisSubmitRequest) -> str:
    canonical = json.dumps(
        {
            "dataset_id": str(dataset_id),
            "analysis_type": payload.analysis_type.value,
            "algorithm_version": payload.algorithm_version,
            "parameters": payload.parameters.model_dump(mode="json"),
        },
        sort_keys=True,
        separators=(",", ":"),
    )
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def _job_response_body(job: AnalysisJob) -> dict[str, object]:
    return {
        "id": str(job.id),
        "dataset_id": str(job.dataset_id),
        "analysis_type": job.analysis_type.value,
        "algorithm_version": job.algorithm_version,
        "status": job.status.value,
        "requested_at": format_utc(job.requested_at),
    }


async def submit_analysis(
    database: Database,
    settings: Settings,
    *,
    context: OrganisationContext,
    dataset_id: UUID,
    payload: AnalysisSubmitRequest,
    idempotency_key: str,
    request_id: str | None = None,
) -> AnalysisJob:
    """Accept an analysis for asynchronous execution.

    The same idempotency key with an identical request replays the original job;
    the same key with a different request is rejected with ``409``.
    """
    tenancy.require_role(context, _WRITE_ROLES)
    organisation_id = context.organisation.id
    request_hash = _request_hash(dataset_id=dataset_id, payload=payload)

    async with database.transaction() as connection:
        dataset = await datasets_repository.get_dataset(
            connection,
            organisation_id=organisation_id,
            dataset_id=dataset_id,
        )
        if dataset is None:
            raise NotFoundError("The requested dataset does not exist.")
        if dataset.status is not DatasetStatus.READY:
            raise ConflictError(
                "Only a validated, ready dataset can be analysed.",
                code="dataset_not_ready",
            )

        reservation = await idempotency_repository.reserve(
            connection,
            organisation_id=organisation_id,
            user_id=context.user.id,
            operation=SUBMIT_OPERATION,
            key=idempotency_key,
            request_hash=request_hash,
            expires_at=_now() + timedelta(seconds=settings.idempotency_ttl_seconds),
        )
        if reservation is None:
            return await _replay_submission(
                connection,
                context=context,
                dataset_id=dataset_id,
                idempotency_key=idempotency_key,
                request_hash=request_hash,
            )

        job = await jobs_repository.create_job(
            connection,
            organisation_id=organisation_id,
            dataset_id=dataset_id,
            requested_by=context.user.id,
            analysis_type=payload.analysis_type,
            algorithm_version=payload.algorithm_version,
            parameters=payload.parameters.model_dump(mode="json"),
            idempotency_key=idempotency_key,
            max_attempts=settings.analysis_max_attempts,
        )
        await outbox_repository.create_event(
            connection,
            organisation_id=organisation_id,
            event_type=ANALYSIS_REQUESTED_EVENT,
            aggregate_type="analysis_job",
            aggregate_id=job.id,
            payload={
                "job_id": str(job.id),
                "dataset_id": str(job.dataset_id),
                "analysis_type": job.analysis_type.value,
            },
        )
        await idempotency_repository.store_response(
            connection,
            record_id=reservation.id,
            response_status=202,
            response_body=_job_response_body(job),
            resource_id=job.id,
        )
        await audit.record_audit_event(
            connection,
            action="analysis.submitted",
            resource_type="analysis_job",
            organisation_id=organisation_id,
            actor_user_id=context.user.id,
            resource_id=job.id,
            request_id=request_id,
            metadata={
                "dataset_id": str(dataset_id),
                "analysis_type": job.analysis_type.value,
                "algorithm_version": job.algorithm_version,
            },
        )
    return job


async def _replay_submission(
    connection: AsyncConnection[dict[str, Any]],
    *,
    context: OrganisationContext,
    dataset_id: UUID,
    idempotency_key: str,
    request_hash: str,
) -> AnalysisJob:
    existing = await idempotency_repository.get_record(
        connection,
        organisation_id=context.organisation.id,
        user_id=context.user.id,
        operation=SUBMIT_OPERATION,
        key=idempotency_key,
    )
    if existing is None or not existing.matches(request_hash) or existing.resource_id is None:
        raise ConflictError(
            "This idempotency key was already used with a different request.",
            code="idempotency_key_reuse",
        )
    job = await jobs_repository.get_job(
        connection,
        organisation_id=context.organisation.id,
        job_id=existing.resource_id,
    )
    if job is None:
        raise ConflictError(
            "This idempotency key was already used with a different request.",
            code="idempotency_key_reuse",
        )
    return job


async def list_analyses(
    database: Database,
    *,
    context: OrganisationContext,
    dataset_id: UUID,
    limit: int,
    cursor: str | None = None,
) -> tuple[list[AnalysisJob], str | None, bool]:
    """Return one page of a dataset's analyses."""
    async with database.connection() as connection:
        dataset = await datasets_repository.get_dataset(
            connection,
            organisation_id=context.organisation.id,
            dataset_id=dataset_id,
        )
        if dataset is None:
            raise NotFoundError("The requested dataset does not exist.")
        items = await jobs_repository.list_jobs(
            connection,
            organisation_id=context.organisation.id,
            dataset_id=dataset_id,
            limit=limit + 1,
            cursor=decode_cursor(cursor) if cursor is not None else None,
        )
    has_more = len(items) > limit
    page = items[:limit]
    next_cursor = (
        encode_cursor(created_at=page[-1].requested_at, id=page[-1].id) if has_more else None
    )
    return page, next_cursor, has_more


async def list_organisation_analyses(
    database: Database,
    *,
    context: OrganisationContext,
    limit: int,
    cursor: str | None = None,
    dataset_id: UUID | None = None,
    study_id: UUID | None = None,
    status: AnalysisJobStatus | None = None,
) -> tuple[list[AnalysisJob], str | None, bool]:
    """Return one page of an organisation's analyses with optional filters.

    The organisation scope is always applied first. A filter that references a
    resource in another organisation simply yields no rows, so the endpoint never
    discloses the existence of a foreign dataset or study.
    """
    async with database.connection() as connection:
        items = await jobs_repository.list_jobs_for_organisation(
            connection,
            organisation_id=context.organisation.id,
            limit=limit + 1,
            dataset_id=dataset_id,
            study_id=study_id,
            status=status,
            cursor=decode_cursor(cursor) if cursor is not None else None,
        )
    has_more = len(items) > limit
    page = items[:limit]
    next_cursor = (
        encode_cursor(created_at=page[-1].requested_at, id=page[-1].id) if has_more else None
    )
    return page, next_cursor, has_more


async def get_analysis(
    database: Database,
    *,
    context: OrganisationContext,
    job_id: UUID,
) -> AnalysisJob:
    """Read a job, hiding other tenants behind a not-found error."""
    async with database.connection() as connection:
        job = await jobs_repository.get_job(
            connection,
            organisation_id=context.organisation.id,
            job_id=job_id,
        )
    if job is None:
        raise NotFoundError("The requested analysis does not exist.")
    return job


async def get_analysis_result(
    database: Database,
    *,
    context: OrganisationContext,
    job_id: UUID,
) -> tuple[AnalysisJob, AnalysisResult]:
    """Return a job and its result, only once the job has succeeded."""
    async with database.connection() as connection:
        job = await jobs_repository.get_job(
            connection,
            organisation_id=context.organisation.id,
            job_id=job_id,
        )
        if job is None:
            raise NotFoundError("The requested analysis does not exist.")
        result = await results_repository.get_result(
            connection,
            organisation_id=context.organisation.id,
            job_id=job_id,
        )
    if job.status is not AnalysisJobStatus.SUCCEEDED:
        if job.status.is_terminal:
            raise ConflictError(
                "No successful result is available for this analysis.",
                code="result_not_available",
            )
        raise ConflictError(
            "The analysis has not finished yet.",
            code="results_not_ready",
        )
    if result is None:
        raise ConflictError(
            "The analysis has not finished yet.",
            code="results_not_ready",
        )
    return job, result


async def cancel_analysis(
    database: Database,
    *,
    context: OrganisationContext,
    job_id: UUID,
    request_id: str | None = None,
) -> AnalysisJob:
    """Cancel a queued job.

    Cancellation is cooperative and only supported before execution begins. A
    running job cannot be reliably interrupted by the initial worker, so it is
    reported with ``409`` rather than a false success (docs/07-api-spec.md §13).
    """
    tenancy.require_role(context, _WRITE_ROLES)
    async with database.transaction() as connection:
        job = await jobs_repository.get_job(
            connection,
            organisation_id=context.organisation.id,
            job_id=job_id,
        )
        if job is None:
            raise NotFoundError("The requested analysis does not exist.")

        if job.status is AnalysisJobStatus.CANCELLED:
            return job
        if job.status is AnalysisJobStatus.RUNNING:
            raise ConflictError(
                "A running analysis cannot be cancelled in the initial implementation.",
                code="analysis_running",
            )
        if job.status is not AnalysisJobStatus.QUEUED:
            raise ConflictError(
                "A completed analysis cannot be cancelled retroactively.",
                code="analysis_not_cancellable",
            )

        cancelled = await jobs_repository.cancel_queued_job(
            connection,
            organisation_id=context.organisation.id,
            job_id=job_id,
        )
        if cancelled is None:
            refreshed = await jobs_repository.get_job(
                connection,
                organisation_id=context.organisation.id,
                job_id=job_id,
            )
            if refreshed is not None and refreshed.status is AnalysisJobStatus.CANCELLED:
                return refreshed
            raise ConflictError(
                "The analysis could not be cancelled in its current state.",
                code="analysis_not_cancellable",
            )
        await audit.record_audit_event(
            connection,
            action="analysis.cancelled",
            resource_type="analysis_job",
            organisation_id=context.organisation.id,
            actor_user_id=context.user.id,
            resource_id=job_id,
            request_id=request_id,
            metadata={"dataset_id": str(job.dataset_id)},
        )
    return cancelled


__all__ = [
    "SUBMIT_OPERATION",
    "cancel_analysis",
    "get_analysis",
    "get_analysis_result",
    "list_analyses",
    "list_organisation_analyses",
    "submit_analysis",
]
