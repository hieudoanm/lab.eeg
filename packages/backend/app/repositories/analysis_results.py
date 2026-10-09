"""Analysis result persistence: explicit SQL and row mapping."""

from __future__ import annotations

from typing import Any
from uuid import UUID

from psycopg import AsyncConnection
from psycopg.types.json import Jsonb

from app.models.analysis import AnalysisResult

_CONNECTION = AsyncConnection[dict[str, Any]]

_RESULT_COLUMNS = (
    "id, organisation_id, job_id, dataset_id, dataset_checksum_sha256, "
    "algorithm_version, result_schema_version, results, provenance, created_at"
)


def _to_result(row: dict[str, Any]) -> AnalysisResult:
    return AnalysisResult(
        id=row["id"],
        organisation_id=row["organisation_id"],
        job_id=row["job_id"],
        dataset_id=row["dataset_id"],
        dataset_checksum_sha256=row["dataset_checksum_sha256"],
        algorithm_version=row["algorithm_version"],
        result_schema_version=row["result_schema_version"],
        results=row["results"] or {},
        provenance=row["provenance"] or {},
        created_at=row["created_at"],
    )


async def create_result(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    job_id: UUID,
    dataset_id: UUID,
    dataset_checksum_sha256: str | None,
    algorithm_version: str,
    result_schema_version: str,
    results: dict[str, Any],
    provenance: dict[str, Any],
) -> AnalysisResult | None:
    """Insert a job's canonical result, returning ``None`` on duplicate.

    ``uq_analysis_results_job_id`` guarantees at most one canonical result per
    job, so a duplicate worker message can never overwrite a committed result.
    """
    cursor = await connection.execute(
        f"""
        INSERT INTO analysis_results (
            organisation_id, job_id, dataset_id, dataset_checksum_sha256,
            algorithm_version, result_schema_version, results, provenance
        )
        VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
        ON CONFLICT (job_id) DO NOTHING
        RETURNING {_RESULT_COLUMNS}
        """,
        (
            organisation_id,
            job_id,
            dataset_id,
            dataset_checksum_sha256,
            algorithm_version,
            result_schema_version,
            Jsonb(results),
            Jsonb(provenance),
        ),
    )
    row = await cursor.fetchone()
    return _to_result(row) if row is not None else None


async def get_result(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    job_id: UUID,
) -> AnalysisResult | None:
    """Return a job's result within the organisation, if it exists."""
    cursor = await connection.execute(
        f"""
        SELECT {_RESULT_COLUMNS}
        FROM analysis_results
        WHERE organisation_id = %s AND job_id = %s
        """,
        (organisation_id, job_id),
    )
    row = await cursor.fetchone()
    return _to_result(row) if row is not None else None


async def get_result_by_job_id(
    connection: _CONNECTION,
    *,
    job_id: UUID,
) -> AnalysisResult | None:
    """Return a job's result without a tenant filter (worker operation)."""
    cursor = await connection.execute(
        f"""
        SELECT {_RESULT_COLUMNS}
        FROM analysis_results
        WHERE job_id = %s
        """,
        (job_id,),
    )
    row = await cursor.fetchone()
    return _to_result(row) if row is not None else None


__all__ = [
    "create_result",
    "get_result",
    "get_result_by_job_id",
]
