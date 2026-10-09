"""Study endpoints."""

from __future__ import annotations

from uuid import UUID

from fastapi import APIRouter, Depends, Query, Request, status

from app.api.dependencies import get_database, get_organisation_context, get_settings
from app.core.config import Settings
from app.db.pool import Database
from app.models.identity import OrganisationContext
from app.models.study import Study, StudyStatus
from app.schemas.study import (
    StudyArchiveResponse,
    StudyCreateRequest,
    StudyListResponse,
    StudyRead,
    StudyUpdateRequest,
)
from app.services import studies as studies_service

router = APIRouter(tags=["studies"])


def _study_read(study: Study) -> StudyRead:
    return StudyRead(
        id=study.id,
        name=study.name,
        description=study.description,
        status=study.status,
        created_at=study.created_at,
        updated_at=study.updated_at,
        archived_at=study.archived_at,
    )


def _request_id(request: Request) -> str | None:
    value = getattr(request.state, "request_id", None)
    return value if isinstance(value, str) else None


@router.post(
    "/organisations/{organisation_id}/studies",
    response_model=StudyRead,
    status_code=status.HTTP_201_CREATED,
    summary="Create a study",
)
async def create_study(
    payload: StudyCreateRequest,
    request: Request,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
) -> StudyRead:
    study = await studies_service.create_study(
        database,
        context=context,
        name=payload.name,
        description=payload.description,
        request_id=_request_id(request),
    )
    return _study_read(study)


@router.get(
    "/organisations/{organisation_id}/studies",
    response_model=StudyListResponse,
    summary="List studies",
)
async def list_studies(
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
    settings: Settings = Depends(get_settings),
    cursor: str | None = Query(default=None),
    search: str | None = Query(default=None, max_length=200),
    status_filter: StudyStatus | None = Query(default=None, alias="status"),
    limit: int | None = Query(default=None, ge=1),
) -> StudyListResponse:
    page_limit = min(limit or settings.default_page_size, settings.max_page_size)
    search_term = search.strip() if search else None
    page, next_cursor, has_more = await studies_service.list_studies(
        database,
        context=context,
        limit=page_limit,
        cursor=cursor,
        search=search_term or None,
        status=status_filter,
    )
    return StudyListResponse(
        items=[_study_read(study) for study in page],
        next_cursor=next_cursor,
        has_more=has_more,
    )


@router.get(
    "/organisations/{organisation_id}/studies/{study_id}",
    response_model=StudyRead,
    summary="Read a study",
)
async def read_study(
    study_id: UUID,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
) -> StudyRead:
    study = await studies_service.get_study(database, context=context, study_id=study_id)
    return _study_read(study)


@router.patch(
    "/organisations/{organisation_id}/studies/{study_id}",
    response_model=StudyRead,
    summary="Update a study",
)
async def update_study(
    study_id: UUID,
    payload: StudyUpdateRequest,
    request: Request,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
) -> StudyRead:
    study = await studies_service.update_study(
        database,
        context=context,
        study_id=study_id,
        changes=payload.model_dump(exclude_unset=True),
        request_id=_request_id(request),
    )
    return _study_read(study)


@router.post(
    "/organisations/{organisation_id}/studies/{study_id}/archive",
    response_model=StudyArchiveResponse,
    summary="Archive a study",
)
async def archive_study(
    study_id: UUID,
    request: Request,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
) -> StudyArchiveResponse:
    study = await studies_service.archive_study(
        database,
        context=context,
        study_id=study_id,
        request_id=_request_id(request),
    )
    assert study.archived_at is not None
    return StudyArchiveResponse(
        id=study.id,
        status=study.status,
        archived_at=study.archived_at,
    )
