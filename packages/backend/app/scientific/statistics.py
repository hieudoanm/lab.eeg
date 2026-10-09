"""Deterministic descriptive statistics for EEG recordings.

Conventions (AGENTS.md §5, docs/06-data-model.md §13):

- Array orientation is ``(channels, samples)``.
- Non-finite samples (NaN, +/-inf) are excluded from every calculation.
- ``valid_sample_count`` reports how many samples were actually used.
- Standard deviation uses the population convention (``ddof=0``).
- A channel with no valid samples is an error, never a zero-filled result.
- Results never claim clinical meaning.
"""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from typing import Any

import numpy as np

Array = np.ndarray[Any, np.dtype[Any]]


class StatisticsInvalidError(Exception):
    """The recording cannot yield valid descriptive statistics.

    ``message`` is safe for display and storage: it never includes filesystem
    paths, stack traces, or internal storage details.
    """


@dataclass(frozen=True, slots=True)
class ChannelStatistics:
    """Descriptive statistics for a single channel."""

    channel_index: int
    channel_name: str
    unit: str | None
    mean: float | None
    standard_deviation: float | None
    minimum: float | None
    maximum: float | None
    peak_to_peak: float | None
    valid_sample_count: int


def compute_descriptive_statistics(
    array: Array,
    *,
    channel_names: Sequence[str],
    channel_units: Sequence[str] | None = None,
    include_mean: bool = True,
    include_standard_deviation: bool = True,
    include_min_max: bool = True,
    include_peak_to_peak: bool = True,
) -> list[ChannelStatistics]:
    """Compute descriptive statistics for every channel of a recording.

    Raises ``StatisticsInvalidError`` when the array is not two-dimensional,
    when channel metadata does not match the array, or when any channel has no
    finite samples.
    """
    if array.ndim != 2:
        raise StatisticsInvalidError(
            "The recording must be a two-dimensional (channels, samples) array."
        )
    channel_count, sample_count = int(array.shape[0]), int(array.shape[1])
    if channel_count < 1:
        raise StatisticsInvalidError("The recording must contain at least one channel.")
    if sample_count < 1:
        raise StatisticsInvalidError("The recording must contain at least one sample per channel.")
    if len(channel_names) != channel_count:
        raise StatisticsInvalidError(
            "The channel metadata does not match the recording: expected "
            f"{channel_count} channel names but received {len(channel_names)}."
        )
    if channel_units is not None and len(channel_units) != channel_count:
        raise StatisticsInvalidError(
            "The channel unit metadata does not match the number of channels."
        )

    values = np.asarray(array, dtype=np.float64)
    statistics: list[ChannelStatistics] = []
    for index in range(channel_count):
        finite = values[index][np.isfinite(values[index])]
        valid_sample_count = int(finite.size)
        if valid_sample_count == 0:
            raise StatisticsInvalidError(f"Channel '{channel_names[index]}' has no valid samples.")
        mean = float(np.mean(finite))
        standard_deviation = float(np.std(finite))
        minimum = float(np.min(finite))
        maximum = float(np.max(finite))
        statistics.append(
            ChannelStatistics(
                channel_index=index,
                channel_name=str(channel_names[index]),
                unit=str(channel_units[index]) if channel_units is not None else None,
                mean=mean if include_mean else None,
                standard_deviation=standard_deviation if include_standard_deviation else None,
                minimum=minimum if include_min_max else None,
                maximum=maximum if include_min_max else None,
                peak_to_peak=(maximum - minimum) if include_peak_to_peak else None,
                valid_sample_count=valid_sample_count,
            )
        )
    return statistics


__all__ = [
    "ChannelStatistics",
    "StatisticsInvalidError",
    "compute_descriptive_statistics",
]
