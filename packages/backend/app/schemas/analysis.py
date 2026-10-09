"""Analysis schemas: submission, job status, and results."""

from __future__ import annotations

from uuid import UUID

from pydantic import Field, model_validator

from app.models.analysis import (
    DESCRIPTIVE_STATISTICS_ALGORITHM_VERSION,
    SUPPORTED_ALGORITHM_VERSIONS,
    AnalysisJobStatus,
    AnalysisType,
)
from app.schemas.common import RequestModel, UtcTimestamp
from app.schemas.pagination import PageResponse


class DescriptiveStatisticsParameters(RequestModel):
    """Which descriptive metrics to compute for each channel."""

    include_mean: bool = True
    include_standard_deviation: bool = True
    include_min_max: bool = True
    include_peak_to_peak: bool = True


class AnalysisSubmitRequest(RequestModel):
    """Submit an analysis for asynchronous execution."""

    analysis_type: AnalysisType = AnalysisType.DESCRIPTIVE_STATISTICS
    algorithm_version: str = Field(
        default=DESCRIPTIVE_STATISTICS_ALGORITHM_VERSION,
        min_length=1,
        max_length=50,
    )
    parameters: DescriptiveStatisticsParameters = Field(
        default_factory=DescriptiveStatisticsParameters
    )

    @model_validator(mode="after")
    def _validate_algorithm_version(self) -> AnalysisSubmitRequest:
        supported = SUPPORTED_ALGORITHM_VERSIONS.get(self.analysis_type, frozenset())
        if self.algorithm_version not in supported:
            available = ", ".join(sorted(supported)) or "none"
            raise ValueError(
                f"Algorithm version '{self.algorithm_version}' is not available for "
                f"analysis type '{self.analysis_type}'. Available versions: {available}."
            )
        return self


class AnalysisError(RequestModel):
    """A safe, classified job failure."""

    code: str
    message: str


class AnalysisJobRead(RequestModel):
    """Current state of an analysis job."""

    id: UUID
    dataset_id: UUID
    analysis_type: AnalysisType
    algorithm_version: str
    status: AnalysisJobStatus
    attempt_count: int
    max_attempts: int
    requested_at: UtcTimestamp
    started_at: UtcTimestamp | None = None
    completed_at: UtcTimestamp | None = None
    error: AnalysisError | None = None


class ChannelStatisticsRead(RequestModel):
    """Descriptive statistics for one channel; disabled metrics are omitted."""

    channel_index: int
    channel_name: str
    unit: str | None = None
    mean: float | None = None
    standard_deviation: float | None = None
    minimum: float | None = None
    maximum: float | None = None
    peak_to_peak: float | None = None
    valid_sample_count: int


class DescriptiveStatisticsResult(RequestModel):
    """The persisted result payload for a descriptive-statistics job."""

    channels: list[ChannelStatisticsRead]


class AnalysisProvenance(RequestModel):
    """Reproducibility metadata stored with every result."""

    input_checksum_sha256: str | None = None
    sampling_frequency_hz: float | None = None
    channel_count: int
    sample_count: int
    parameters: DescriptiveStatisticsParameters


class AnalysisResultRead(RequestModel):
    """A successful analysis result and its provenance."""

    analysis_id: UUID
    dataset_id: UUID
    algorithm_version: str
    result_schema_version: str
    created_at: UtcTimestamp
    channels: list[ChannelStatisticsRead]
    provenance: AnalysisProvenance


class AnalysisListResponse(PageResponse[AnalysisJobRead]):
    """Paginated list of a dataset's analyses."""


__all__ = [
    "AnalysisError",
    "AnalysisJobRead",
    "AnalysisListResponse",
    "AnalysisProvenance",
    "AnalysisResultRead",
    "AnalysisSubmitRequest",
    "ChannelStatisticsRead",
    "DescriptiveStatisticsParameters",
    "DescriptiveStatisticsResult",
]
