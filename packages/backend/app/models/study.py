"""Study domain model."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from enum import StrEnum
from uuid import UUID


class StudyStatus(StrEnum):
    """Lifecycle states for a study."""

    ACTIVE = "active"
    ARCHIVED = "archived"


@dataclass(frozen=True, slots=True)
class Study:
    """A research study owned by an organisation."""

    id: UUID
    organisation_id: UUID
    name: str
    description: str | None
    status: StudyStatus
    created_by: UUID
    created_at: datetime
    updated_at: datetime
    archived_at: datetime | None

    @property
    def is_archived(self) -> bool:
        return self.status is StudyStatus.ARCHIVED
