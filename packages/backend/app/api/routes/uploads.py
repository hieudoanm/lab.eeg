"""Upload session endpoints.

Phase 5 uses a backend-mediated upload for local development: the API returns a
relative ``upload_url`` pointing at the authenticated ``PUT .../content``
endpoint, which stores the recording bytes in private object storage. Production
may switch to a signed direct-to-storage flow later without changing the session
or finalisation contract.
"""

from __future__ import annotations

from uuid import UUID

from fastapi import APIRouter, Depends, Request, Response, status

from app.api.dependencies import (
    get_database,
    get_organisation_context,
    get_request_id,
    get_settings,
    get_storage,
)
from app.api.routes.datasets import _dataset_read
from app.core.config import Settings
from app.db.pool import Database
from app.models.identity import OrganisationContext
from app.models.upload_session import UploadSession, UploadSessionStatus
from app.schemas.upload import (
    UploadFinaliseResponse,
    UploadSessionCreateRequest,
    UploadSessionRead,
)
from app.services import uploads as uploads_service
from app.storage.base import Storage

router = APIRouter(tags=["uploads"])


def _content_url(organisation_id: UUID, dataset_id: UUID, session_id: UUID) -> str:
    return (
        f"/api/v1/organisations/{organisation_id}/datasets/{dataset_id}"
        f"/upload-sessions/{session_id}/content"
    )


def _upload_read(
    session: UploadSession,
    *,
    organisation_id: UUID,
    dataset_id: UUID,
) -> UploadSessionRead:
    return UploadSessionRead(
        upload_session_id=session.id,
        status=session.status,
        upload_method="backend",
        upload_url=_content_url(organisation_id, dataset_id, session.id),
        expires_at=session.expires_at,
    )


@router.post(
    "/organisations/{organisation_id}/datasets/{dataset_id}/upload-sessions",
    response_model=UploadSessionRead,
    status_code=status.HTTP_201_CREATED,
    summary="Create an upload session for a dataset",
)
async def create_upload_session(
    dataset_id: UUID,
    payload: UploadSessionCreateRequest,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
    settings: Settings = Depends(get_settings),
    storage: Storage = Depends(get_storage),
    request_id: str | None = Depends(get_request_id),
) -> UploadSessionRead:
    session = await uploads_service.create_upload_session(
        database,
        storage,
        settings,
        context=context,
        dataset_id=dataset_id,
        payload=payload,
        request_id=request_id,
    )
    return _upload_read(session, organisation_id=context.organisation.id, dataset_id=dataset_id)


@router.put(
    "/organisations/{organisation_id}/datasets/{dataset_id}/upload-sessions/{session_id}/content",
    status_code=status.HTTP_204_NO_CONTENT,
    summary="Store the recording bytes for an upload session",
)
async def store_upload_content(
    dataset_id: UUID,
    session_id: UUID,
    request: Request,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
    settings: Settings = Depends(get_settings),
    storage: Storage = Depends(get_storage),
) -> Response:
    body = await request.body()
    await uploads_service.store_upload_content(
        database,
        storage,
        settings,
        context=context,
        dataset_id=dataset_id,
        session_id=session_id,
        body=body,
    )
    return Response(status_code=status.HTTP_204_NO_CONTENT)


@router.post(
    "/organisations/{organisation_id}/datasets/{dataset_id}/upload-sessions/{session_id}/finalise",
    response_model=UploadFinaliseResponse,
    summary="Finalise and validate a recording upload",
)
async def finalise_upload(
    dataset_id: UUID,
    session_id: UUID,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
    settings: Settings = Depends(get_settings),
    storage: Storage = Depends(get_storage),
    request_id: str | None = Depends(get_request_id),
) -> UploadFinaliseResponse:
    result = await uploads_service.finalise_upload(
        database,
        storage,
        settings,
        context=context,
        dataset_id=dataset_id,
        session_id=session_id,
        request_id=request_id,
    )
    return UploadFinaliseResponse(
        upload_session_id=result.session.id,
        dataset=_dataset_read(result.dataset),
        status=UploadSessionStatus.COMPLETED,
    )
