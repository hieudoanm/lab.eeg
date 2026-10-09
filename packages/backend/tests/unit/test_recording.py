"""Unit tests for .npy recording validation (no database required)."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import numpy as np
import pytest
from app.scientific import ChannelManifest, RecordingInvalidError, validate_recording


def _write_npy(
    path: Path, data: np.ndarray[Any, np.dtype[Any]], *, allow_pickle: bool = False
) -> Path:
    np.save(path, data, allow_pickle=allow_pickle)
    return path


def _manifest(*, frequency: float = 256.0, channels: list[str] | None = None) -> ChannelManifest:
    return ChannelManifest(
        sampling_frequency_hz=frequency,
        channel_names=channels or ["F3", "F4", "C3", "C4"],
        channel_units=None,
    )


def test_valid_recording_passes(tmp_path: Path) -> None:
    path = _write_npy(tmp_path / "rec.npy", np.zeros((4, 512), dtype=np.float32))
    recording = validate_recording(
        path, manifest=_manifest(), max_channels=256, max_sample_count=1_000_000
    )
    assert recording.channel_count == 4
    assert recording.sample_count == 512
    assert recording.sampling_frequency_hz == 256.0
    assert recording.duration_seconds == pytest.approx(2.0)
    assert recording.dtype_name == "float32"


def test_integer_and_bool_dtypes_are_accepted(tmp_path: Path) -> None:
    for dtype in (np.int16, np.uint8, np.bool_):
        path = _write_npy(tmp_path / "rec.npy", np.zeros((2, 16), dtype=dtype))
        recording = validate_recording(
            path, manifest=_manifest(channels=["A", "B"]), max_channels=4, max_sample_count=1_000
        )
        assert recording.channel_count == 2


def test_one_dimensional_array_is_rejected(tmp_path: Path) -> None:
    path = _write_npy(tmp_path / "rec.npy", np.zeros(64, dtype=np.float32))
    with pytest.raises(RecordingInvalidError):
        validate_recording(path, manifest=_manifest(), max_channels=256, max_sample_count=1_000_000)


def test_three_dimensional_array_is_rejected(tmp_path: Path) -> None:
    path = _write_npy(tmp_path / "rec.npy", np.zeros((2, 3, 4), dtype=np.float32))
    with pytest.raises(RecordingInvalidError):
        validate_recording(path, manifest=_manifest(), max_channels=256, max_sample_count=1_000_000)


def test_non_numeric_dtype_is_rejected(tmp_path: Path) -> None:
    path = _write_npy(tmp_path / "rec.npy", np.zeros((2, 4), dtype="<U8"))
    with pytest.raises(RecordingInvalidError):
        validate_recording(path, manifest=_manifest(), max_channels=256, max_sample_count=1_000_000)


def test_pickled_object_array_is_rejected(tmp_path: Path) -> None:
    path = _write_npy(
        tmp_path / "rec.npy",
        np.array([object()] * 8, dtype=object).reshape(2, 4),
        allow_pickle=True,
    )
    with pytest.raises(RecordingInvalidError):
        validate_recording(path, manifest=_manifest(), max_channels=256, max_sample_count=1_000_000)


def test_garbage_file_is_rejected(tmp_path: Path) -> None:
    path = tmp_path / "rec.npy"
    path.write_bytes(b"not-a-npy-file")
    with pytest.raises(RecordingInvalidError):
        validate_recording(path, manifest=_manifest(), max_channels=256, max_sample_count=1_000_000)


def test_too_many_channels_is_rejected(tmp_path: Path) -> None:
    path = _write_npy(tmp_path / "rec.npy", np.zeros((4, 16), dtype=np.float32))
    with pytest.raises(RecordingInvalidError):
        validate_recording(
            path,
            manifest=_manifest(channels=["A", "B", "C", "D"]),
            max_channels=2,
            max_sample_count=1_000_000,
        )


def test_too_many_samples_is_rejected(tmp_path: Path) -> None:
    path = _write_npy(tmp_path / "rec.npy", np.zeros((2, 16), dtype=np.float32))
    with pytest.raises(RecordingInvalidError):
        validate_recording(
            path, manifest=_manifest(channels=["A", "B"]), max_channels=4, max_sample_count=8
        )


def test_channel_metadata_mismatch_is_rejected(tmp_path: Path) -> None:
    path = _write_npy(tmp_path / "rec.npy", np.zeros((4, 16), dtype=np.float32))
    manifest = ChannelManifest(
        sampling_frequency_hz=256.0, channel_names=["only-one"], channel_units=None
    )
    with pytest.raises(RecordingInvalidError):
        validate_recording(path, manifest=manifest, max_channels=16, max_sample_count=1_000_000)


def test_missing_or_invalid_sampling_frequency_is_rejected(tmp_path: Path) -> None:
    path = _write_npy(tmp_path / "rec.npy", np.zeros((2, 16), dtype=np.float32))
    for frequency in (None, 0.0, -1.0):
        manifest = ChannelManifest(
            sampling_frequency_hz=frequency, channel_names=["A", "B"], channel_units=None
        )
        with pytest.raises(RecordingInvalidError):
            validate_recording(path, manifest=manifest, max_channels=4, max_sample_count=1_000)
