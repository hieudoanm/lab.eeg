"""Unit tests for bounded signal-window extraction."""

from __future__ import annotations

from pathlib import Path

import numpy as np
import pytest
from app.scientific.recording import ChannelManifest
from app.scientific.signals import (
    SignalWindow,
    SignalWindowInvalidError,
    extract_signal_window,
)

FREQUENCY = 256.0
NAMES = ["F3", "F4"]
UNITS = ["uV", "uV"]


def _write(path: Path, values: np.ndarray) -> Path:
    np.save(path, values, allow_pickle=False)
    return path


def _manifest() -> ChannelManifest:
    return ChannelManifest(
        sampling_frequency_hz=FREQUENCY,
        channel_names=list(NAMES),
        channel_units=list(UNITS),
    )


def _extract(path: Path, **overrides: object) -> SignalWindow:
    options: dict[str, object] = {
        "manifest": _manifest(),
        "start_seconds": 0.0,
        "end_seconds": 1.0,
        "channel_indices": [0],
        "max_channels": 256,
        "max_sample_count": 10_000_000,
        "max_window_seconds": 300.0,
        "max_points_per_channel": 20_000,
    }
    options.update(overrides)
    return extract_signal_window(path, **options)  # type: ignore[arg-type]


def test_extracts_a_window_with_a_shared_time_axis(tmp_path: Path) -> None:
    values = np.vstack(
        [np.arange(FREQUENCY, dtype=np.float64), np.arange(FREQUENCY, dtype=np.float64) * 2]
    )
    path = _write(tmp_path / "recording.npy", values)

    window = _extract(path, start_seconds=0.0, end_seconds=1.0, channel_indices=[0, 1])

    assert window.sample_start == 0
    assert window.sample_end == 256
    assert window.downsampled is False
    assert window.downsample_factor == 1
    assert window.points_per_channel == 256
    assert window.time_seconds[0] == 0.0
    assert window.time_seconds[1] == pytest.approx(1.0 / FREQUENCY)
    assert [channel.channel_index for channel in window.channels] == [0, 1]
    assert window.channels[0].channel_name == "F3"
    assert window.channels[0].unit == "uV"
    assert window.channels[1].samples[3] == pytest.approx(6.0)


def test_decimates_when_the_window_exceeds_the_point_budget(tmp_path: Path) -> None:
    values = np.zeros((2, 1024), dtype=np.float64)
    path = _write(tmp_path / "recording.npy", values)

    window = _extract(path, end_seconds=4.0, channel_indices=[0], max_points_per_channel=100)

    assert window.downsampled is True
    assert window.downsample_factor == 11
    assert window.points_per_channel == 94
    assert window.time_seconds[1] == pytest.approx(11.0 / FREQUENCY)


def test_maps_non_finite_samples_to_none(tmp_path: Path) -> None:
    values = np.array([[1.0, np.nan, np.inf, -np.inf, 5.0]], dtype=np.float64)
    path = _write(tmp_path / "recording.npy", values)
    manifest = ChannelManifest(
        sampling_frequency_hz=FREQUENCY,
        channel_names=["F3"],
        channel_units=["uV"],
    )

    window = _extract(
        path,
        manifest=manifest,
        end_seconds=5.0 / FREQUENCY,
        channel_indices=[0],
    )

    assert window.channels[0].samples == [1.0, None, None, None, 5.0]


def test_duplicate_channels_are_collapsed(tmp_path: Path) -> None:
    values = np.zeros((2, 256), dtype=np.float64)
    path = _write(tmp_path / "recording.npy", values)

    window = _extract(path, channel_indices=[1, 1, 0])

    assert [channel.channel_index for channel in window.channels] == [1, 0]


@pytest.mark.parametrize(
    ("overrides", "reason"),
    [
        ({"start_seconds": -1.0}, "negative start"),
        ({"start_seconds": 1.0, "end_seconds": 1.0}, "empty window"),
        ({"end_seconds": 301.0}, "window beyond the limit"),
        ({"end_seconds": 2.0}, "window beyond the recording"),
        ({"channel_indices": [5]}, "channel out of range"),
        ({"channel_indices": [-1]}, "negative channel"),
        ({"channel_indices": []}, "no channels"),
    ],
)
def test_invalid_windows_are_rejected(
    tmp_path: Path, overrides: dict[str, object], reason: str
) -> None:
    values = np.zeros((2, 256), dtype=np.float64)
    path = _write(tmp_path / "recording.npy", values)

    with pytest.raises(SignalWindowInvalidError):
        _extract(path, **overrides)
