"""Deterministic scientific computations.

Every computation records its inputs (dataset checksum, algorithm version,
parameters, result-schema version) and never claims clinical meaning. Array
orientation is ``(channels, samples)`` everywhere.
"""

from app.scientific.recording import (
    ChannelManifest,
    RecordingInvalidError,
    RecordingMetadata,
    validate_recording,
)
from app.scientific.statistics import (
    ChannelStatistics,
    StatisticsInvalidError,
    compute_descriptive_statistics,
)

__all__ = [
    "ChannelManifest",
    "ChannelStatistics",
    "RecordingInvalidError",
    "RecordingMetadata",
    "StatisticsInvalidError",
    "compute_descriptive_statistics",
    "validate_recording",
]
