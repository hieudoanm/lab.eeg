"""Organisation dashboard aggregates."""

from __future__ import annotations

from dataclasses import dataclass

from app.db.pool import Database
from app.models.identity import OrganisationContext
from app.repositories import summary as summary_repository


@dataclass(frozen=True, slots=True)
class OrganisationSummary:
    """Counts backing the organisation dashboard."""

    study_count: int
    active_study_count: int
    dataset_count: int
    ready_dataset_count: int
    analysis_count: int
    analyses_by_status: dict[str, int]


async def get_organisation_summary(
    database: Database,
    *,
    context: OrganisationContext,
) -> OrganisationSummary:
    """Return resource counts for the caller's organisation."""
    organisation_id = context.organisation.id
    async with database.connection() as connection:
        counts = await summary_repository.count_resources(
            connection, organisation_id=organisation_id
        )
        by_status = await summary_repository.count_analyses_by_status(
            connection, organisation_id=organisation_id
        )
    return OrganisationSummary(
        study_count=counts["study_count"],
        active_study_count=counts["active_study_count"],
        dataset_count=counts["dataset_count"],
        ready_dataset_count=counts["ready_dataset_count"],
        analysis_count=counts["analysis_count"],
        analyses_by_status=by_status,
    )


__all__ = ["OrganisationSummary", "get_organisation_summary"]
