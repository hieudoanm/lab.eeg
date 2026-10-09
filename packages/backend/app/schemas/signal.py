"""Signal window schemas for the interactive chart display endpoint."""

from __future__ import annotations

from uuid import UUID

from app.schemas.common import RequestModel


class SignalChannelRead(RequestModel):
    """A decimated channel signal within a requested window."""

    channel_index: int
    channel_name: str
    unit: str | None = None
    samples: list[float | None]


class SignalWindowRead(RequestModel):
    """A bounded, decimated slice of a recording for display.

    ``samples`` align with ``time_seconds``; non-finite samples are reported as
    ``null`` so a chart renders a gap, and ``downsampled`` records whether the
    samples were decimated for transport.
    """

    dataset_id: UUID
    sampling_frequency_hz: float
    channel_count: int
    sample_count: int
    duration_seconds: float
    start_seconds: float
    end_seconds: float
    sample_start: int
    sample_end: int
    channel_indices: list[int]
    points_per_channel: int
    downsampled: bool
    downsample_factor: int
    time_seconds: list[float]
    channels: list[SignalChannelRead]
