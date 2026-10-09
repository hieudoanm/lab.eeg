"""Bounded, read-only signal-window extraction for chart display.

A signal window is requested in seconds, converted to sample indices using the
recording's sampling frequency, and decimated to a bounded number of points so
that a browser never receives an unbounded payload. Decimation is a display
concern only: it is never an input to analysis and never mutates the stored
recording, which is opened memory-mapped and read-only.

Non-finite samples are reported as ``None`` so a chart can render a gap rather
than fabricate a value.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np

from app.scientific.recording import (
    ChannelManifest,
    RecordingMetadata,
    validate_recording,
)

#: Tolerance (in seconds) applied when checking a window against the duration.
_DURATION_TOLERANCE_SECONDS = 1e-9


class SignalWindowInvalidError(Exception):
    """The requested signal window is invalid or cannot be served safely.

    ``message`` is deliberately safe for display and must never include
    filesystem paths, stack traces, or internal storage details.
    """


@dataclass(frozen=True, slots=True)
class ChannelSignal:
    """The decimated samples for one channel within the requested window."""

    channel_index: int
    channel_name: str
    unit: str | None
    samples: list[float | None]


@dataclass(frozen=True, slots=True)
class SignalWindow:
    """A validated, decimated slice of a recording."""

    metadata: RecordingMetadata
    start_seconds: float
    end_seconds: float
    sample_start: int
    sample_end: int
    downsampled: bool
    downsample_factor: int
    points_per_channel: int
    time_seconds: list[float]
    channels: list[ChannelSignal]


def extract_signal_window(
    path: Path,
    *,
    manifest: ChannelManifest,
    start_seconds: float,
    end_seconds: float,
    channel_indices: list[int],
    max_channels: int,
    max_sample_count: int,
    max_window_seconds: float,
    max_points_per_channel: int,
) -> SignalWindow:
    """Validate, slice, and decimate a recording window for display.

    Raises ``SignalWindowInvalidError`` for a bad request and propagates
    ``RecordingInvalidError`` when the stored object is not a valid recording.
    """
    metadata: RecordingMetadata = validate_recording(
        path,
        manifest=manifest,
        max_channels=max_channels,
        max_sample_count=max_sample_count,
    )

    if not (np.isfinite(start_seconds) and np.isfinite(end_seconds)):
        raise SignalWindowInvalidError("The window bounds must be finite numbers.")
    if start_seconds < 0:
        raise SignalWindowInvalidError("The window start must not be negative.")
    if end_seconds <= start_seconds:
        raise SignalWindowInvalidError("The window end must be greater than its start.")
    window_seconds = end_seconds - start_seconds
    if window_seconds > max_window_seconds:
        raise SignalWindowInvalidError(
            f"The requested window exceeds the {max_window_seconds:g} second limit."
        )
    if end_seconds > metadata.duration_seconds + _DURATION_TOLERANCE_SECONDS:
        raise SignalWindowInvalidError("The requested window extends beyond the recording.")

    frequency = metadata.sampling_frequency_hz
    start_sample = max(0, min(int(np.floor(start_seconds * frequency)), metadata.sample_count))
    end_sample = max(
        start_sample,
        min(int(np.ceil(end_seconds * frequency)), metadata.sample_count),
    )
    if end_sample <= start_sample:
        raise SignalWindowInvalidError("The requested window contains no samples.")

    selected = _select_channels(channel_indices, metadata.channel_count)

    span = end_sample - start_sample
    factor = max(1, int(np.ceil(span / max(1, max_points_per_channel))))
    indices = np.arange(start_sample, end_sample, factor, dtype=np.int64)

    names = list(manifest.channel_names)
    units = list(manifest.channel_units) if manifest.channel_units else None

    memmap = np.load(path, mmap_mode="r", allow_pickle=False)
    try:
        if memmap.ndim != 2:
            raise SignalWindowInvalidError(
                "The recording must be a two-dimensional (channels, samples) array."
            )
        channels: list[ChannelSignal] = []
        for channel_index in selected:
            window = np.asarray(
                memmap[channel_index, start_sample:end_sample][::factor],
                dtype=np.float64,
            )
            channels.append(
                ChannelSignal(
                    channel_index=channel_index,
                    channel_name=_channel_name(names, channel_index),
                    unit=_channel_unit(units, channel_index),
                    samples=_finite_samples(window),
                )
            )
    finally:
        close = getattr(memmap, "close", None)
        if callable(close):
            close()

    return SignalWindow(
        metadata=metadata,
        start_seconds=start_seconds,
        end_seconds=end_seconds,
        sample_start=start_sample,
        sample_end=end_sample,
        downsampled=factor > 1,
        downsample_factor=factor,
        points_per_channel=int(indices.size),
        time_seconds=(indices / frequency).tolist(),
        channels=channels,
    )


def _select_channels(channel_indices: list[int], channel_count: int) -> list[int]:
    """De-duplicate a channel selection while preserving order, validating range."""
    selected: list[int] = []
    seen: set[int] = set()
    for channel_index in channel_indices:
        if channel_index < 0 or channel_index >= channel_count:
            raise SignalWindowInvalidError("A selected channel is outside the recording.")
        if channel_index not in seen:
            seen.add(channel_index)
            selected.append(channel_index)
    if not selected:
        raise SignalWindowInvalidError("At least one channel must be selected.")
    return selected


def _channel_name(names: list[str], channel_index: int) -> str:
    if channel_index < len(names):
        return names[channel_index]
    return f"channel_{channel_index}"


def _channel_unit(units: list[str] | None, channel_index: int) -> str | None:
    if units is not None and channel_index < len(units):
        return units[channel_index]
    return None


def _finite_samples(values: np.ndarray[Any, np.dtype[Any]]) -> list[float | None]:
    """Convert samples to floats, mapping non-finite values to ``None``."""
    return [float(value) if np.isfinite(value) else None for value in values.tolist()]


__all__ = [
    "ChannelSignal",
    "SignalWindow",
    "SignalWindowInvalidError",
    "extract_signal_window",
]
