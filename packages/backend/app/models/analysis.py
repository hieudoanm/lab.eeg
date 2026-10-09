"""Analysis domain models.

Analysis runs asynchronously: the API persists a durable job (and its outbox
event) and returns immediately, while a worker claims the job, loads the
recording, computes a deterministic result, and commits it. These dataclasses
are the boundary between repositories and the application layer.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from enum import StrEnum
from typing import Any
from uuid import UUID


class AnalysisType(StrEnum):
    """Supported analysis identifiers."""

    DESCRIPTIVE_STATISTICS = "descriptive_statistics"


class AnalysisJobStatus(StrEnum):
    """Durable analysis job lifecycle."""

    QUEUED = "queued"
    RUNNING = "running"
    SUCCEEDED = "succeeded"
    FAILED = "failed"
    CANCELLED = "cancelled"

    @property
    def is_terminal(self) -> bool:
        return self in {
            AnalysisJobStatus.SUCCEEDED,
            AnalysisJobStatus.FAILED,
            AnalysisJobStatus.CANCELLED,
        }


#: Algorithm versions available per analysis type.
SUPPORTED_ALGORITHM_VERSIONS: dict[AnalysisType, frozenset[str]] = {
    AnalysisType.DESCRIPTIVE_STATISTICS: frozenset({"1.0.0"}),
}

#: Version stamped onto every persisted descriptive-statistics result.
DESCRIPTIVE_STATISTICS_ALGORITHM_VERSION = "1.0.0"
RESULT_SCHEMA_VERSION = "1.0.0"

#: Outbox event emitted when a job is accepted for execution.
ANALYSIS_REQUESTED_EVENT = "analysis.requested"

#: Celery task that executes an analysis job.
ANALYSIS_TASK_NAME = "eeg-lab.run_analysis"

#: Mapping from outbox event type to the Celery task that handles it.
ANALYSIS_EVENT_TASK_NAMES: dict[str, str] = {
    ANALYSIS_REQUESTED_EVENT: ANALYSIS_TASK_NAME,
}


@dataclass(frozen=True, slots=True)
class AnalysisJob:
    """A durable asynchronous analysis job and its state."""

    id: UUID
    organisation_id: UUID
    dataset_id: UUID
    requested_by: UUID
    analysis_type: AnalysisType
    algorithm_version: str
    parameters: dict[str, Any]
    status: AnalysisJobStatus
    attempt_count: int
    max_attempts: int
    error_code: str | None
    error_message: str | None
    idempotency_key: str | None
    requested_at: datetime
    started_at: datetime | None
    completed_at: datetime | None
    lease_expires_at: datetime | None
    created_at: datetime
    updated_at: datetime

    @property
    def is_terminal(self) -> bool:
        return self.status.is_terminal


@dataclass(frozen=True, slots=True)
class AnalysisResult:
    """A persisted, validated analysis result and its provenance."""

    id: UUID
    organisation_id: UUID
    job_id: UUID
    dataset_id: UUID
    dataset_checksum_sha256: str | None
    algorithm_version: str
    result_schema_version: str
    results: dict[str, Any]
    provenance: dict[str, Any]
    created_at: datetime


@dataclass(frozen=True, slots=True)
class OutboxEvent:
    """A durable dispatch record for background work."""

    id: UUID
    organisation_id: UUID | None
    event_type: str
    aggregate_type: str
    aggregate_id: UUID
    payload: dict[str, Any]
    created_at: datetime
    published_at: datetime | None
    attempt_count: int
    next_attempt_at: datetime | None
    last_error_code: str | None

    @property
    def is_published(self) -> bool:
        return self.published_at is not None


@dataclass(frozen=True, slots=True)
class IdempotencyRecord:
    """A durable record deduplicating a state-changing request."""

    id: UUID
    organisation_id: UUID
    user_id: UUID
    operation: str
    key: str
    request_hash: str
    response_status: int | None
    response_body: dict[str, Any] | None
    resource_id: UUID | None
    created_at: datetime
    expires_at: datetime

    def matches(self, request_hash: str) -> bool:
        """Whether the recorded request is identical to a repeated one."""
        return self.request_hash == request_hash


__all__ = [
    "ANALYSIS_EVENT_TASK_NAMES",
    "ANALYSIS_REQUESTED_EVENT",
    "ANALYSIS_TASK_NAME",
    "DESCRIPTIVE_STATISTICS_ALGORITHM_VERSION",
    "RESULT_SCHEMA_VERSION",
    "SUPPORTED_ALGORITHM_VERSIONS",
    "AnalysisJob",
    "AnalysisJobStatus",
    "AnalysisResult",
    "AnalysisType",
    "IdempotencyRecord",
    "OutboxEvent",
]
