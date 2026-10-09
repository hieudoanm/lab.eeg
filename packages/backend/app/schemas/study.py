"""Study schemas: request and response models."""

from __future__ import annotations

from uuid import UUID

from pydantic import Field, field_validator

from app.models.study import StudyStatus
from app.schemas.common import RequestModel, UtcTimestamp
from app.schemas.pagination import PageResponse


class StudyCreateRequest(RequestModel):
    """Request to create a study."""

    name: str = Field(min_length=1, max_length=200)
    description: str | None = Field(default=None, max_length=2000)

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


class StudyUpdateRequest(RequestModel):
    """Partial update of a study."""

    name: str | None = Field(default=None, min_length=1, max_length=200)
    description: str | None = Field(default=None, max_length=2000)

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


class StudyRead(RequestModel):
    """Study representation returned to API clients."""

    id: UUID
    name: str
    description: str | None
    status: StudyStatus
    created_at: UtcTimestamp
    updated_at: UtcTimestamp
    archived_at: UtcTimestamp | None = None


class StudyListResponse(PageResponse[StudyRead]):
    """Paginated list of studies."""


class StudyArchiveResponse(RequestModel):
    """Response returned after archiving a study."""

    id: UUID
    status: StudyStatus
    archived_at: UtcTimestamp
