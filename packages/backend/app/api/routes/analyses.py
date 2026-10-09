"""Analysis endpoints.

Submission is asynchronous: the API persists the job and its outbox event
transactionally and returns ``202 Accepted``. Status and results are read from
persisted state, and cancellation is cooperative (queued jobs only).
"""

from __future__ import annotations

from uuid import UUID

from fastapi import APIRouter, Depends, Header, Query, status

from app.api.dependencies import (
    get_database,
    get_organisation_context,
    get_request_id,
    get_settings,
)
from app.core.config import Settings
from app.db.pool import Database
from app.models.analysis import AnalysisJob, AnalysisJobStatus, AnalysisResult
from app.models.identity import OrganisationContext
from app.schemas.analysis import (
    AnalysisError,
    AnalysisJobRead,
    AnalysisListResponse,
    AnalysisProvenance,
    AnalysisResultRead,
    AnalysisSubmitRequest,
    DescriptiveStatisticsResult,
)
from app.services import analyses as analyses_service

router = APIRouter(tags=["analyses"])

_SAFE_FAILURE_MESSAGE = "The analysis could not be completed."


def _job_read(job: AnalysisJob) -> AnalysisJobRead:
    error = None
    if job.status is AnalysisJobStatus.FAILED and job.error_code is not None:
        error = AnalysisError(
            code=job.error_code,
            message=job.error_message or _SAFE_FAILURE_MESSAGE,
        )
    return AnalysisJobRead(
        id=job.id,
        dataset_id=job.dataset_id,
        analysis_type=job.analysis_type,
        algorithm_version=job.algorithm_version,
        status=job.status,
        attempt_count=job.attempt_count,
        max_attempts=job.max_attempts,
        requested_at=job.requested_at,
        started_at=job.started_at,
        completed_at=job.completed_at,
        error=error,
    )


def _result_read(job: AnalysisJob, result: AnalysisResult) -> AnalysisResultRead:
    payload = DescriptiveStatisticsResult.model_validate(result.results)
    provenance = AnalysisProvenance.model_validate(result.provenance)
    return AnalysisResultRead(
        analysis_id=job.id,
        dataset_id=result.dataset_id,
        algorithm_version=result.algorithm_version,
        result_schema_version=result.result_schema_version,
        created_at=result.created_at,
        channels=payload.channels,
        provenance=provenance,
    )


@router.post(
    "/organisations/{organisation_id}/datasets/{dataset_id}/analyses",
    response_model=AnalysisJobRead,
    status_code=status.HTTP_202_ACCEPTED,
    summary="Submit an analysis for asynchronous execution",
)
async def submit_analysis(
    dataset_id: UUID,
    payload: AnalysisSubmitRequest,
    idempotency_key: str = Header(alias="Idempotency-Key", min_length=1, max_length=255),
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
    settings: Settings = Depends(get_settings),
    request_id: str | None = Depends(get_request_id),
) -> AnalysisJobRead:
    job = await analyses_service.submit_analysis(
        database,
        settings,
        context=context,
        dataset_id=dataset_id,
        payload=payload,
        idempotency_key=idempotency_key,
        request_id=request_id,
    )
    return _job_read(job)


@router.get(
    "/organisations/{organisation_id}/datasets/{dataset_id}/analyses",
    response_model=AnalysisListResponse,
    summary="List a dataset's analyses",
)
async def list_analyses(
    dataset_id: UUID,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
    settings: Settings = Depends(get_settings),
    cursor: str | None = Query(default=None),
    limit: int | None = Query(default=None, ge=1),
) -> AnalysisListResponse:
    page, next_cursor, has_more = await analyses_service.list_analyses(
        database,
        context=context,
        dataset_id=dataset_id,
        limit=min(limit or settings.default_page_size, settings.max_page_size),
        cursor=cursor,
    )
    return AnalysisListResponse(
        items=[_job_read(job) for job in page],
        next_cursor=next_cursor,
        has_more=has_more,
    )


@router.get(
    "/organisations/{organisation_id}/analyses",
    response_model=AnalysisListResponse,
    summary="List an organisation's analyses",
)
async def list_organisation_analyses(
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
    settings: Settings = Depends(get_settings),
    cursor: str | None = Query(default=None),
    limit: int | None = Query(default=None, ge=1),
    dataset_id: UUID | None = Query(default=None),
    study_id: UUID | None = Query(default=None),
    status_filter: AnalysisJobStatus | None = Query(default=None, alias="status"),
) -> AnalysisListResponse:
    page, next_cursor, has_more = await analyses_service.list_organisation_analyses(
        database,
        context=context,
        limit=min(limit or settings.default_page_size, settings.max_page_size),
        cursor=cursor,
        dataset_id=dataset_id,
        study_id=study_id,
        status=status_filter,
    )
    return AnalysisListResponse(
        items=[_job_read(job) for job in page],
        next_cursor=next_cursor,
        has_more=has_more,
    )


@router.get(
    "/organisations/{organisation_id}/analyses/{analysis_id}",
    response_model=AnalysisJobRead,
    summary="Read an analysis job's status",
)
async def read_analysis(
    analysis_id: UUID,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
) -> AnalysisJobRead:
    job = await analyses_service.get_analysis(database, context=context, job_id=analysis_id)
    return _job_read(job)


@router.get(
    "/organisations/{organisation_id}/analyses/{analysis_id}/results",
    response_model=AnalysisResultRead,
    summary="Retrieve a successful analysis result",
)
async def read_analysis_result(
    analysis_id: UUID,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
) -> AnalysisResultRead:
    job, result = await analyses_service.get_analysis_result(
        database, context=context, job_id=analysis_id
    )
    return _result_read(job, result)


@router.post(
    "/organisations/{organisation_id}/analyses/{analysis_id}/cancel",
    response_model=AnalysisJobRead,
    summary="Request cancellation of a queued analysis",
)
async def cancel_analysis(
    analysis_id: UUID,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
    request_id: str | None = Depends(get_request_id),
) -> AnalysisJobRead:
    job = await analyses_service.cancel_analysis(
        database,
        context=context,
        job_id=analysis_id,
        request_id=request_id,
    )
    return _job_read(job)
