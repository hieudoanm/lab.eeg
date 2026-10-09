"""Dataset schemas: request and response models."""

from __future__ import annotations

from uuid import UUID

from pydantic import Field, field_validator

from app.models.dataset import DatasetFormat, DatasetStatus
from app.schemas.common import RequestModel, UtcTimestamp
from app.schemas.pagination import PageResponse


class DatasetCreateRequest(RequestModel):
    """Register a dataset before upload."""

    name: str = Field(min_length=1, max_length=200)
    description: str | None = Field(default=None, max_length=2000)
    format: DatasetFormat = DatasetFormat.NPY
    metadata: dict[str, object] = Field(default_factory=dict)
    source_description: str | None = Field(default=None, max_length=2000)

    @field_validator("name", mode="before")
    @classmethod
    def _strip_name(cls, value: object) -> object:
        return value.strip() if isinstance(value, str) else value

    @field_validator("description", mode="before")
    @classmethod
    def _strip_description(cls, value: object) -> object:
        if isinstance(value, str):
            stripped = value.strip()
            return stripped if stripped else None
        return value

    @field_validator("source_description", mode="before")
    @classmethod
    def _strip_source(cls, value: object) -> object:
        if isinstance(value, str):
            stripped = value.strip()
            return stripped if stripped else None
        return value


class DatasetRead(RequestModel):
    """Dataset metadata returned to API clients."""

    id: UUID
    study_id: UUID
    name: str
    description: str | None
    format: DatasetFormat
    status: DatasetStatus
    sampling_frequency_hz: float | None = None
    channel_count: int | None = None
    sample_count: int | None = None
    duration_seconds: float | None = None
    channel_names: list[str] | None = None
    channel_units: list[str] | None = None
    checksum_sha256: str | None = None
    file_size_bytes: int | None = None
    source_description: str | None = None
    created_at: UtcTimestamp
    updated_at: UtcTimestamp
    validated_at: UtcTimestamp | None = None


class DatasetListResponse(PageResponse[DatasetRead]):
    """Paginated list of datasets within a study."""
