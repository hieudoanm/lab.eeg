"""Analysis job persistence: explicit SQL and row mapping."""

from __future__ import annotations

from datetime import datetime
from typing import Any
from uuid import UUID

from psycopg import AsyncConnection
from psycopg.types.json import Jsonb

from app.core.pagination import Cursor
from app.models.analysis import AnalysisJob, AnalysisJobStatus, AnalysisType

_CONNECTION = AsyncConnection[dict[str, Any]]

_JOB_COLUMNS = (
    "id, organisation_id, dataset_id, requested_by, analysis_type, algorithm_version, "
    "parameters, status, attempt_count, max_attempts, error_code, error_message, "
    "idempotency_key, requested_at, started_at, completed_at, lease_expires_at, "
    "created_at, updated_at"
)

_JOB_COLUMNS_ALIASED = ", ".join(f"j.{column.strip()}" for column in _JOB_COLUMNS.split(","))


def _to_job(row: dict[str, Any]) -> AnalysisJob:
    return AnalysisJob(
        id=row["id"],
        organisation_id=row["organisation_id"],
        dataset_id=row["dataset_id"],
        requested_by=row["requested_by"],
        analysis_type=AnalysisType(row["analysis_type"]),
        algorithm_version=row["algorithm_version"],
        parameters=row["parameters"] or {},
        status=AnalysisJobStatus(row["status"]),
        attempt_count=row["attempt_count"],
        max_attempts=row["max_attempts"],
        error_code=row["error_code"],
        error_message=row["error_message"],
        idempotency_key=row["idempotency_key"],
        requested_at=row["requested_at"],
        started_at=row["started_at"],
        completed_at=row["completed_at"],
        lease_expires_at=row["lease_expires_at"],
        created_at=row["created_at"],
        updated_at=row["updated_at"],
    )


async def create_job(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    dataset_id: UUID,
    requested_by: UUID,
    analysis_type: AnalysisType,
    algorithm_version: str,
    parameters: dict[str, Any],
    idempotency_key: str | None,
    max_attempts: int,
) -> AnalysisJob:
    """Insert a job in the ``queued`` state."""
    cursor = await connection.execute(
        f"""
        INSERT INTO analysis_jobs (
            organisation_id, dataset_id, requested_by, analysis_type,
            algorithm_version, parameters, idempotency_key, max_attempts
        )
        VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
        RETURNING {_JOB_COLUMNS}
        """,
        (
            organisation_id,
            dataset_id,
            requested_by,
            analysis_type.value,
            algorithm_version,
            Jsonb(parameters),
            idempotency_key,
            max_attempts,
        ),
    )
    row = await cursor.fetchone()
    assert row is not None
    return _to_job(row)


async def get_job(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    job_id: UUID,
) -> AnalysisJob | None:
    """Return a job within the given organisation, if it exists."""
    cursor = await connection.execute(
        f"""
        SELECT {_JOB_COLUMNS}
        FROM analysis_jobs
        WHERE organisation_id = %s AND id = %s
        """,
        (organisation_id, job_id),
    )
    row = await cursor.fetchone()
    return _to_job(row) if row is not None else None


async def get_job_by_id(
    connection: _CONNECTION,
    *,
    job_id: UUID,
) -> AnalysisJob | None:
    """Return a job by identifier without a tenant filter (worker operation)."""
    cursor = await connection.execute(
        f"""
        SELECT {_JOB_COLUMNS}
        FROM analysis_jobs
        WHERE id = %s
        """,
        (job_id,),
    )
    row = await cursor.fetchone()
    return _to_job(row) if row is not None else None


async def list_jobs(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    dataset_id: UUID,
    limit: int,
    cursor: Cursor | None = None,
) -> list[AnalysisJob]:
    """List a dataset's jobs using keyset pagination on ``requested_at``."""
    conditions = ["organisation_id = %s", "dataset_id = %s"]
    params: list[Any] = [organisation_id, dataset_id]
    if cursor is not None:
        conditions.append("(requested_at, id) < (%s, %s)")
        params.extend([cursor.created_at, cursor.id])
    params.append(limit)
    query = f"""
        SELECT {_JOB_COLUMNS}
        FROM analysis_jobs
        WHERE {" AND ".join(conditions)}
        ORDER BY requested_at DESC, id DESC
        LIMIT %s
    """
    result = await connection.execute(query, params)
    return [_to_job(row) async for row in result]


async def list_jobs_for_organisation(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    limit: int,
    dataset_id: UUID | None = None,
    study_id: UUID | None = None,
    status: AnalysisJobStatus | None = None,
    cursor: Cursor | None = None,
) -> list[AnalysisJob]:
    """List an organisation's jobs with optional dataset, study, and status filters.

    Ordering is ``requested_at DESC, id DESC`` and keyset pagination is applied
    after the organisation scope. ``study_id`` joins the owning dataset.
    """
    join = ""
    conditions = ["j.organisation_id = %s"]
    params: list[Any] = [organisation_id]
    if study_id is not None:
        join = "JOIN datasets d ON d.id = j.dataset_id"
        conditions.append("d.study_id = %s")
        params.append(study_id)
    if dataset_id is not None:
        conditions.append("j.dataset_id = %s")
        params.append(dataset_id)
    if status is not None:
        conditions.append("j.status = %s")
        params.append(status.value)
    if cursor is not None:
        conditions.append("(j.requested_at, j.id) < (%s, %s)")
        params.extend([cursor.created_at, cursor.id])
    params.append(limit)
    query = f"""
        SELECT {_JOB_COLUMNS_ALIASED}
        FROM analysis_jobs j
        {join}
        WHERE {" AND ".join(conditions)}
        ORDER BY j.requested_at DESC, j.id DESC
        LIMIT %s
    """
    result = await connection.execute(query, params)
    return [_to_job(row) async for row in result]


async def claim_job(
    connection: _CONNECTION,
    *,
    job_id: UUID,
    lease_expires_at: datetime,
) -> AnalysisJob | None:
    """Atomically transition ``queued`` → ``running`` and take a lease.

    Bumps ``attempt_count`` so retries are bounded. Returns ``None`` when the
    job is missing or no longer queued (for example a duplicate delivery of a
    completed message, or a cancellation that arrived first).
    """
    cursor = await connection.execute(
        f"""
        UPDATE analysis_jobs
        SET status = 'running',
            attempt_count = attempt_count + 1,
            started_at = now(),
            lease_expires_at = %s,
            error_code = NULL,
            error_message = NULL,
            updated_at = now()
        WHERE id = %s AND status = 'queued'
        RETURNING {_JOB_COLUMNS}
        """,
        (lease_expires_at, job_id),
    )
    row = await cursor.fetchone()
    return _to_job(row) if row is not None else None


async def complete_job(
    connection: _CONNECTION,
    *,
    job_id: UUID,
) -> AnalysisJob | None:
    """Transition ``running`` → ``succeeded``."""
    cursor = await connection.execute(
        f"""
        UPDATE analysis_jobs
        SET status = 'succeeded',
            completed_at = now(),
            lease_expires_at = NULL,
            updated_at = now()
        WHERE id = %s AND status = 'running'
        RETURNING {_JOB_COLUMNS}
        """,
        (job_id,),
    )
    row = await cursor.fetchone()
    return _to_job(row) if row is not None else None


async def fail_job(
    connection: _CONNECTION,
    *,
    job_id: UUID,
    error_code: str,
    error_message: str,
) -> AnalysisJob | None:
    """Transition a non-terminal job to the terminal ``failed`` state."""
    cursor = await connection.execute(
        f"""
        UPDATE analysis_jobs
        SET status = 'failed',
            error_code = %s,
            error_message = %s,
            completed_at = now(),
            lease_expires_at = NULL,
            updated_at = now()
        WHERE id = %s AND status IN ('queued', 'running')
        RETURNING {_JOB_COLUMNS}
        """,
        (error_code, error_message, job_id),
    )
    row = await cursor.fetchone()
    return _to_job(row) if row is not None else None


async def requeue_job(
    connection: _CONNECTION,
    *,
    job_id: UUID,
) -> AnalysisJob | None:
    """Transition a stale/permitted-retry ``running`` job back to ``queued``."""
    cursor = await connection.execute(
        f"""
        UPDATE analysis_jobs
        SET status = 'queued',
            started_at = NULL,
            lease_expires_at = NULL,
            updated_at = now()
        WHERE id = %s AND status = 'running'
        RETURNING {_JOB_COLUMNS}
        """,
        (job_id,),
    )
    row = await cursor.fetchone()
    return _to_job(row) if row is not None else None


async def cancel_queued_job(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    job_id: UUID,
) -> AnalysisJob | None:
    """Cancel a queued job within the organisation, before execution begins."""
    cursor = await connection.execute(
        f"""
        UPDATE analysis_jobs
        SET status = 'cancelled',
            completed_at = now(),
            updated_at = now()
        WHERE organisation_id = %s AND id = %s AND status = 'queued'
        RETURNING {_JOB_COLUMNS}
        """,
        (organisation_id, job_id),
    )
    row = await cursor.fetchone()
    return _to_job(row) if row is not None else None


async def claim_stale_jobs(
    connection: _CONNECTION,
    *,
    now: datetime,
    limit: int,
) -> list[AnalysisJob]:
    """Lock and return running jobs whose lease has expired.

    Crosses all organisations because stale-job recovery is a maintenance
    operation. Rows are locked with ``FOR UPDATE SKIP LOCKED`` so concurrent
    recoveries never contend for the same job.
    """
    cursor = await connection.execute(
        f"""
        SELECT {_JOB_COLUMNS}
        FROM analysis_jobs
        WHERE status = 'running'
          AND lease_expires_at IS NOT NULL
          AND lease_expires_at <= %s
        ORDER BY lease_expires_at
        LIMIT %s
        FOR UPDATE SKIP LOCKED
        """,
        (now, limit),
    )
    rows = await cursor.fetchall()
    return [_to_job(row) for row in rows]


__all__ = [
    "cancel_queued_job",
    "claim_job",
    "claim_stale_jobs",
    "complete_job",
    "create_job",
    "fail_job",
    "get_job",
    "get_job_by_id",
    "list_jobs",
    "list_jobs_for_organisation",
    "requeue_job",
]
