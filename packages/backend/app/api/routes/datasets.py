"""Dataset endpoints."""

from __future__ import annotations

from uuid import UUID

from fastapi import APIRouter, Depends, Query, Request, status

from app.api.dependencies import get_database, get_organisation_context, get_settings
from app.core.config import Settings
from app.db.pool import Database
from app.models.dataset import Dataset
from app.models.identity import OrganisationContext
from app.schemas.dataset import DatasetCreateRequest, DatasetListResponse, DatasetRead
from app.services import datasets as datasets_service

router = APIRouter(tags=["datasets"])


def _dataset_read(dataset: Dataset) -> DatasetRead:
    return DatasetRead(
        id=dataset.id,
        study_id=dataset.study_id,
        name=dataset.name,
        description=dataset.description,
        format=dataset.format,
        status=dataset.status,
        sampling_frequency_hz=dataset.sampling_frequency_hz,
        channel_count=dataset.channel_count,
        sample_count=dataset.sample_count,
        duration_seconds=dataset.duration_seconds,
        channel_names=dataset.channel_names,
        channel_units=dataset.channel_units,
        checksum_sha256=dataset.checksum_sha256,
        file_size_bytes=dataset.file_size_bytes,
        source_description=dataset.source_description,
        created_at=dataset.created_at,
        updated_at=dataset.updated_at,
        validated_at=dataset.validated_at,
    )


def _request_id(request: Request) -> str | None:
    value = getattr(request.state, "request_id", None)
    return value if isinstance(value, str) else None


@router.post(
    "/organisations/{organisation_id}/studies/{study_id}/datasets",
    response_model=DatasetRead,
    status_code=status.HTTP_201_CREATED,
    summary="Register dataset metadata before upload",
)
async def register_dataset(
    study_id: UUID,
    payload: DatasetCreateRequest,
    request: Request,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
) -> DatasetRead:
    dataset = await datasets_service.register_dataset(
        database,
        context=context,
        study_id=study_id,
        name=payload.name,
        description=payload.description,
        dataset_format=payload.format,
        source_description=payload.source_description,
        metadata=payload.metadata,
        request_id=_request_id(request),
    )
    return _dataset_read(dataset)


@router.get(
    "/organisations/{organisation_id}/studies/{study_id}/datasets",
    response_model=DatasetListResponse,
    summary="List datasets within a study",
)
async def list_datasets(
    study_id: UUID,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
    settings: Settings = Depends(get_settings),
    cursor: str | None = Query(default=None),
    limit: int | None = Query(default=None, ge=1),
) -> DatasetListResponse:
    page, next_cursor, has_more = await datasets_service.list_datasets(
        database,
        context=context,
        study_id=study_id,
        limit=min(limit or settings.default_page_size, settings.max_page_size),
        cursor=cursor,
    )
    return DatasetListResponse(
        items=[_dataset_read(dataset) for dataset in page],
        next_cursor=next_cursor,
        has_more=has_more,
    )


@router.get(
    "/organisations/{organisation_id}/datasets/{dataset_id}",
    response_model=DatasetRead,
    summary="Read dataset metadata",
)
async def read_dataset(
    dataset_id: UUID,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
) -> DatasetRead:
    dataset = await datasets_service.get_dataset(database, context=context, dataset_id=dataset_id)
    return _dataset_read(dataset)
