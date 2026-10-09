"""Dataset domain model."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime
from enum import StrEnum
from typing import Any
from uuid import UUID


class DatasetStatus(StrEnum):
    """Upload and validation lifecycle for a dataset."""

    PENDING_UPLOAD = "pending_upload"
    VALIDATING = "validating"
    READY = "ready"
    REJECTED = "rejected"
    FAILED = "failed"
    ARCHIVED = "archived"


class DatasetFormat(StrEnum):
    """Supported recording formats."""

    NPY = "npy"


@dataclass(frozen=True, slots=True)
class Dataset:
    """A registered recording and its metadata.

    ``storage_key`` identifies the private object and is intentionally excluded
    from API responses; the API layer maps only approved fields onto its schemas.
    """

    id: UUID
    organisation_id: UUID
    study_id: UUID
    name: str
    description: str | None
    status: DatasetStatus
    format: DatasetFormat
    storage_key: str | None
    checksum_sha256: str | None
    file_size_bytes: int | None
    sampling_frequency_hz: float | None
    channel_count: int | None
    sample_count: int | None
    duration_seconds: float | None
    channel_names: list[str] | None
    channel_units: list[str] | None
    source_description: str | None
    uploaded_by: UUID
    created_at: datetime
    updated_at: datetime
    validated_at: datetime | None
    archived_at: datetime | None
    metadata: dict[str, Any] = field(default_factory=dict)
