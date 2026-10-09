"""Dataset persistence: explicit SQL and row mapping."""

from __future__ import annotations

from typing import Any
from uuid import UUID

from psycopg import AsyncConnection
from psycopg.types.json import Jsonb

from app.core.pagination import Cursor
from app.models.dataset import Dataset, DatasetFormat, DatasetStatus

_CONNECTION = AsyncConnection[dict[str, Any]]

_DATASET_COLUMNS = (
    "id, organisation_id, study_id, name, description, status, format, storage_key, "
    "checksum_sha256, file_size_bytes, sampling_frequency_hz, channel_count, sample_count, "
    "duration_seconds, channel_names, channel_units, source_description, uploaded_by, "
    "created_at, updated_at, validated_at, archived_at, metadata"
)


def _to_dataset(row: dict[str, Any]) -> Dataset:
    return Dataset(
        id=row["id"],
        organisation_id=row["organisation_id"],
        study_id=row["study_id"],
        name=row["name"],
        description=row["description"],
        status=DatasetStatus(row["status"]),
        format=DatasetFormat(row["format"]),
        storage_key=row["storage_key"],
        checksum_sha256=row["checksum_sha256"],
        file_size_bytes=row["file_size_bytes"],
        sampling_frequency_hz=row["sampling_frequency_hz"],
        channel_count=row["channel_count"],
        sample_count=row["sample_count"],
        duration_seconds=row["duration_seconds"],
        channel_names=row["channel_names"],
        channel_units=row["channel_units"],
        source_description=row["source_description"],
        uploaded_by=row["uploaded_by"],
        created_at=row["created_at"],
        updated_at=row["updated_at"],
        validated_at=row["validated_at"],
        archived_at=row["archived_at"],
        metadata=row["metadata"] or {},
    )


async def create_dataset(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    study_id: UUID,
    name: str,
    description: str | None,
    dataset_format: DatasetFormat,
    source_description: str | None,
    metadata: dict[str, Any],
    uploaded_by: UUID,
) -> Dataset:
    """Register a dataset in ``pending_upload`` state."""
    cursor = await connection.execute(
        f"""
        INSERT INTO datasets (
            organisation_id, study_id, name, description, format,
            source_description, metadata, uploaded_by
        )
        VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
        RETURNING {_DATASET_COLUMNS}
        """,
        (
            organisation_id,
            study_id,
            name,
            description,
            dataset_format.value,
            source_description,
            Jsonb(metadata),
            uploaded_by,
        ),
    )
    row = await cursor.fetchone()
    assert row is not None
    return _to_dataset(row)


async def get_dataset(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    dataset_id: UUID,
) -> Dataset | None:
    """Return a dataset within the given organisation, if it exists."""
    cursor = await connection.execute(
        f"""
        SELECT {_DATASET_COLUMNS}
        FROM datasets
        WHERE organisation_id = %s AND id = %s
        """,
        (organisation_id, dataset_id),
    )
    row = await cursor.fetchone()
    return _to_dataset(row) if row is not None else None


async def list_datasets(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    study_id: UUID,
    limit: int,
    cursor: Cursor | None = None,
) -> list[Dataset]:
    """List datasets for a study using keyset pagination."""
    conditions = ["organisation_id = %s", "study_id = %s"]
    params: list[Any] = [organisation_id, study_id]
    if cursor is not None:
        conditions.append("(created_at, id) < (%s, %s)")
        params.extend([cursor.created_at, cursor.id])
    params.append(limit)
    query = f"""
        SELECT {_DATASET_COLUMNS}
        FROM datasets
        WHERE {" AND ".join(conditions)}
        ORDER BY created_at DESC, id DESC
        LIMIT %s
    """
    result = await connection.execute(query, params)
    return [_to_dataset(row) async for row in result]


async def mark_dataset_validated(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    dataset_id: UUID,
    storage_key: str,
    checksum_sha256: str,
    file_size_bytes: int,
    sampling_frequency_hz: float,
    channel_count: int,
    sample_count: int,
    duration_seconds: float,
    channel_names: list[str],
    channel_units: list[str] | None,
    metadata: dict[str, Any],
) -> Dataset | None:
    """Transition a dataset to ``ready`` after successful validation.

    Only datasets that are not archived or already ready can become ready, so a
    duplicate finalisation cannot overwrite a committed recording.
    """
    cursor = await connection.execute(
        f"""
        UPDATE datasets
        SET status = 'ready',
            storage_key = %s,
            checksum_sha256 = %s,
            file_size_bytes = %s,
            sampling_frequency_hz = %s,
            channel_count = %s,
            sample_count = %s,
            duration_seconds = %s,
            channel_names = %s,
            channel_units = %s,
            metadata = %s,
            validated_at = now(),
            updated_at = now()
        WHERE organisation_id = %s
          AND id = %s
          AND status IN ('pending_upload', 'validating', 'rejected', 'failed')
        RETURNING {_DATASET_COLUMNS}
        """,
        (
            storage_key,
            checksum_sha256,
            file_size_bytes,
            sampling_frequency_hz,
            channel_count,
            sample_count,
            duration_seconds,
            Jsonb(channel_names),
            Jsonb(channel_units) if channel_units is not None else None,
            Jsonb(metadata),
            organisation_id,
            dataset_id,
        ),
    )
    row = await cursor.fetchone()
    return _to_dataset(row) if row is not None else None


async def mark_dataset_rejected(
    connection: _CONNECTION,
    *,
    organisation_id: UUID,
    dataset_id: UUID,
) -> Dataset | None:
    """Transition a dataset to ``rejected`` after failed validation.

    The rejected dataset may be re-uploaded through a new session.
    """
    cursor = await connection.execute(
        f"""
        UPDATE datasets
        SET status = 'rejected', updated_at = now()
        WHERE organisation_id = %s
          AND id = %s
          AND status IN ('pending_upload', 'validating', 'failed')
        RETURNING {_DATASET_COLUMNS}
        """,
        (organisation_id, dataset_id),
    )
    row = await cursor.fetchone()
    return _to_dataset(row) if row is not None else None
