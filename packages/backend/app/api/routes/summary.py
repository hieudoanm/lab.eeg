"""Organisation dashboard summary endpoint."""

from __future__ import annotations

from fastapi import APIRouter, Depends

from app.api.dependencies import get_database, get_organisation_context
from app.db.pool import Database
from app.models.identity import OrganisationContext
from app.schemas.summary import AnalysesByStatus, OrganisationSummaryRead
from app.services import summary as summary_service

router = APIRouter(tags=["summary"])


@router.get(
    "/organisations/{organisation_id}/summary",
    response_model=OrganisationSummaryRead,
    summary="Read organisation dashboard counts",
)
async def read_organisation_summary(
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
) -> OrganisationSummaryRead:
    summary = await summary_service.get_organisation_summary(database, context=context)
    return OrganisationSummaryRead(
        study_count=summary.study_count,
        active_study_count=summary.active_study_count,
        dataset_count=summary.dataset_count,
        ready_dataset_count=summary.ready_dataset_count,
        analysis_count=summary.analysis_count,
        analyses_by_status=AnalysesByStatus(**summary.analyses_by_status),
    )
