"""Aggregate counts backing the organisation dashboard."""

from __future__ import annotations

from typing import Any
from uuid import UUID

from psycopg import AsyncConnection

from app.models.analysis import AnalysisJobStatus

_CONNECTION = AsyncConnection[dict[str, Any]]

_COUNT_COLUMNS = (
    "study_count",
    "active_study_count",
    "dataset_count",
    "ready_dataset_count",
    "analysis_count",
)


async def count_resources(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
) -> dict[str, int]:
    """Return resource counts for an organisation.

    Every subquery filters by the organisation first; datasets and studies are
    never counted across tenants.
    """
    cursor = await connection.execute(
        """
        SELECT
            (SELECT count(*) FROM studies
                WHERE organisation_id = %s) AS study_count,
            (SELECT count(*) FROM studies
                WHERE organisation_id = %s AND status = 'active') AS active_study_count,
            (SELECT count(*) FROM datasets
                WHERE organisation_id = %s) AS dataset_count,
            (SELECT count(*) FROM datasets
                WHERE organisation_id = %s AND status = 'ready') AS ready_dataset_count,
            (SELECT count(*) FROM analysis_jobs
                WHERE organisation_id = %s) AS analysis_count
        """,
        (organisation_id,) * len(_COUNT_COLUMNS),
    )
    row = await cursor.fetchone()
    assert row is not None
    return {column: int(row[column]) for column in _COUNT_COLUMNS}


async def count_analyses_by_status(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
) -> dict[str, int]:
    """Return analysis counts keyed by status, including zero-count statuses."""
    counts = {status.value: 0 for status in AnalysisJobStatus}
    result = await connection.execute(
        """
        SELECT status, count(*) AS total
        FROM analysis_jobs
        WHERE organisation_id = %s
        GROUP BY status
        """,
        (organisation_id,),
    )
    async for row in result:
        counts[str(row["status"])] = int(row["total"])
    return counts


__all__ = ["count_analyses_by_status", "count_resources"]
