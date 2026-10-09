"""Deterministic NumPy recording validation for the initial ``.npy`` format.

The recording contract:

- Orientation is ``(channels, samples)``.
- Supported dtypes are numeric (floating, signed or unsigned integer, or bool).
- Channel metadata in the upload manifest must match the first dimension.
- Resource limits (channel count, sample count, file size) are enforced before
  an uploaded object is accepted.
- Pickle-based object loading is always disabled for untrusted arrays.

Validation reads only the array header, so a pathological recording does not get
materialised into API memory during finalisation.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import numpy as np

_SUPPORTED_DTYPE_KINDS = frozenset({"f", "i", "u", "b"})


class RecordingInvalidError(Exception):
    """The recording violates the documented format or validation rules.

    ``message`` is deliberately safe for display and storage. It must never
    include filesystem paths, stack traces, or internal storage details.
    """


@dataclass(frozen=True, slots=True)
class ChannelManifest:
    """Channel metadata supplied at upload time and verified against the array."""

    sampling_frequency_hz: float | None = None
    channel_names: list[str] = field(default_factory=list)
    channel_units: list[str] | None = None


@dataclass(frozen=True, slots=True)
class RecordingMetadata:
    """Validated properties extracted from a recording and its manifest."""

    channel_count: int
    sample_count: int
    sampling_frequency_hz: float
    duration_seconds: float
    dtype_name: str


def read_recording_path(path: Path) -> tuple[tuple[int, ...], np.dtype[Any]]:
    """Return ``(shape, dtype)`` for an ``.npy`` file without loading its data.

    The array is opened with memory-mapped access and pickle loading disabled,
    so object arrays (a common pickle-based attack vector) are rejected.
    """
    try:
        memmap = np.load(path, mmap_mode="r", allow_pickle=False)
        try:
            shape = tuple(int(dimension) for dimension in memmap.shape)
            dtype = memmap.dtype
        finally:
            close = getattr(memmap, "close", None)
            if callable(close):
                close()
    except ValueError as exc:
        raise RecordingInvalidError(
            "The recording is not a valid .npy file or uses an unsupported format."
        ) from exc
    return shape, dtype


def _require_numeric(dtype: np.dtype[Any]) -> None:
    if dtype.kind not in _SUPPORTED_DTYPE_KINDS:
        raise RecordingInvalidError("The recording must use a numeric data type.")


def validate_recording(
    path: Path,
    *,
    manifest: ChannelManifest,
    max_channels: int,
    max_sample_count: int,
) -> RecordingMetadata:
    """Validate an ``.npy`` recording and return its metadata.

    Raises ``RecordingInvalidError`` with a safe message when the recording
    violates the format contract or resource limits.
    """
    shape, dtype = read_recording_path(path)
    _require_numeric(dtype)

    if len(shape) != 2:
        raise RecordingInvalidError(
            "The recording must be a two-dimensional (channels, samples) array."
        )
    channel_count, sample_count = shape
    if channel_count < 1:
        raise RecordingInvalidError("The recording must contain at least one channel.")
    if channel_count > max_channels:
        raise RecordingInvalidError(
            f"The recording has {channel_count} channels; the limit is {max_channels}."
        )
    if sample_count < 1:
        raise RecordingInvalidError("The recording must contain at least one sample per channel.")
    if sample_count > max_sample_count:
        raise RecordingInvalidError(
            f"The recording has {sample_count} samples per channel; "
            f"the limit is {max_sample_count}."
        )

    if manifest.channel_names and len(manifest.channel_names) != channel_count:
        raise RecordingInvalidError(
            "The channel metadata does not match the recording: expected "
            f"{channel_count} channel names but received {len(manifest.channel_names)}."
        )

    frequency = manifest.sampling_frequency_hz
    if frequency is None:
        raise RecordingInvalidError("A sampling frequency must be supplied for .npy recordings.")
    if not (float(frequency) > 0.0) or not np.isfinite(frequency):
        raise RecordingInvalidError("The sampling frequency must be finite and greater than zero.")

    duration_seconds = sample_count / frequency
    return RecordingMetadata(
        channel_count=channel_count,
        sample_count=sample_count,
        sampling_frequency_hz=frequency,
        duration_seconds=duration_seconds,
        dtype_name=str(dtype),
    )


__all__ = [
    "ChannelManifest",
    "RecordingInvalidError",
    "RecordingMetadata",
    "validate_recording",
]
