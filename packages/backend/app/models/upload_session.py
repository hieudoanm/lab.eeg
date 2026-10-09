"""Upload session domain model."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import UTC, datetime
from enum import StrEnum
from typing import Any
from uuid import UUID


class UploadSessionStatus(StrEnum):
    """Lifecycle states for an upload session."""

    CREATED = "created"
    UPLOADING = "uploading"
    UPLOADED = "uploaded"
    FINALISING = "finalising"
    COMPLETED = "completed"
    FAILED = "failed"
    EXPIRED = "expired"


@dataclass(frozen=True, slots=True)
class UploadSession:
    """An in-progress upload attempt for a registered dataset.

    ``object_key`` is a server-generated private storage reference and must
    never be exposed through the API. ``manifest`` carries the validated
    metadata supplied at session creation that is not embedded in the array.
    """

    id: UUID
    organisation_id: UUID
    dataset_id: UUID
    created_by: UUID
    status: UploadSessionStatus
    object_key: str | None
    expected_size_bytes: int | None
    expected_checksum_sha256: str | None
    expires_at: datetime
    finalised_at: datetime | None
    created_at: datetime
    updated_at: datetime
    manifest: dict[str, Any] = field(default_factory=dict)

    @property
    def is_expired(self) -> bool:
        """Whether the session has passed its upload deadline."""
        return datetime.now(UTC) >= self.expires_at
