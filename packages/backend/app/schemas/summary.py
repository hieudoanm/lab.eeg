"""Organisation dashboard summary schema."""

from __future__ import annotations

from app.schemas.common import RequestModel


class AnalysesByStatus(RequestModel):
    """Analysis job counts grouped by lifecycle status."""

    queued: int = 0
    running: int = 0
    succeeded: int = 0
    failed: int = 0
    cancelled: int = 0


class OrganisationSummaryRead(RequestModel):
    """Aggregate counts for an organisation's dashboard."""

    study_count: int
    active_study_count: int
    dataset_count: int
    ready_dataset_count: int
    analysis_count: int
    analyses_by_status: AnalysesByStatus
