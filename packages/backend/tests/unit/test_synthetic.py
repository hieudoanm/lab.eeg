"""Unit tests for deterministic synthetic recording generation (no database)."""

from __future__ import annotations

import json
from dataclasses import replace
from pathlib import Path

import numpy as np
import pytest
from app.scientific import ChannelManifest, validate_recording
from app.scientific.synthetic import (
    SyntheticRecordingConfig,
    channel_names,
    generate_recording,
    main,
    write_recording,
)


def test_generation_is_deterministic_for_a_fixed_seed() -> None:
    config = SyntheticRecordingConfig(channel_count=4, sample_count=256, seed=42)

    first = generate_recording(config)
    second = generate_recording(config)

    np.testing.assert_array_equal(first.array, second.array)
    assert first.manifest == second.manifest


def test_different_seeds_produce_different_signals() -> None:
    base = SyntheticRecordingConfig(channel_count=2, sample_count=128, pattern="noise")
    first = generate_recording(base).array
    second = generate_recording(replace(base, seed=1)).array

    assert not np.array_equal(first, second)


def test_array_has_expected_shape_and_dtype() -> None:
    recording = generate_recording(
        SyntheticRecordingConfig(channel_count=3, sample_count=64, pattern="mixed")
    )

    assert recording.array.shape == (3, 64)
    assert recording.array.dtype == np.float32


def test_channel_names_are_zero_padded_and_counted() -> None:
    assert channel_names(3) == ["EEG 001", "EEG 002", "EEG 003"]


def test_nonfinite_pattern_introduces_non_finite_samples() -> None:
    recording = generate_recording(
        SyntheticRecordingConfig(channel_count=2, sample_count=64, pattern="nonfinite")
    )

    assert not np.isfinite(recording.array).all()


def test_constant_pattern_uses_exact_channel_values() -> None:
    recording = generate_recording(
        SyntheticRecordingConfig(channel_count=3, sample_count=16, pattern="constant")
    )

    np.testing.assert_array_equal(recording.array[0], np.ones(16, dtype=np.float32))
    np.testing.assert_array_equal(recording.array[2], np.full(16, 3.0, dtype=np.float32))


def test_generated_recording_passes_validation(tmp_path: Path) -> None:
    recording = generate_recording(
        SyntheticRecordingConfig(channel_count=4, sample_count=512, sampling_frequency_hz=250.0)
    )
    output = tmp_path / "recording.npy"
    write_recording(recording, output)

    validated = validate_recording(
        output,
        manifest=ChannelManifest(
            sampling_frequency_hz=recording.manifest.sampling_frequency_hz,
            channel_names=recording.manifest.channel_names,
            channel_units=recording.manifest.channel_units,
        ),
        max_channels=256,
        max_sample_count=1_000_000,
    )

    assert validated.channel_count == 4
    assert validated.sample_count == 512
    assert validated.sampling_frequency_hz == 250.0
    assert validated.duration_seconds == pytest.approx(512 / 250.0)


def test_write_recording_emits_npy_and_manifest(tmp_path: Path) -> None:
    recording = generate_recording(SyntheticRecordingConfig(channel_count=2, sample_count=32))
    output = tmp_path / "nested" / "recording.npy"
    manifest_path = tmp_path / "nested" / "recording.json"

    write_recording(recording, output, manifest_path=manifest_path)

    assert np.load(output, allow_pickle=False).shape == (2, 32)
    payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    assert payload["channel_names"] == ["EEG 001", "EEG 002"]
    assert payload["channel_units"] == ["uV", "uV"]
    assert payload["sampling_frequency_hz"] == 256.0
    assert payload["pattern"] == "mixed"


def test_configuration_rejects_invalid_values() -> None:
    with pytest.raises(ValueError):
        SyntheticRecordingConfig(channel_count=0)
    with pytest.raises(ValueError):
        SyntheticRecordingConfig(sample_count=0)
    with pytest.raises(ValueError):
        SyntheticRecordingConfig(sampling_frequency_hz=0.0)
    with pytest.raises(ValueError):
        SyntheticRecordingConfig(amplitude=-1.0)


def test_cli_writes_recording_and_manifest(tmp_path: Path) -> None:
    output = tmp_path / "cli.npy"

    exit_code = main(
        [
            "--channels",
            "2",
            "--samples",
            "48",
            "--seed",
            "7",
            "--pattern",
            "sinusoid",
            "--output",
            str(output),
        ]
    )

    assert exit_code == 0
    assert np.load(output, allow_pickle=False).shape == (2, 48)
    assert output.with_suffix(".json").exists()


def test_cli_can_skip_the_manifest(tmp_path: Path) -> None:
    output = tmp_path / "cli.npy"

    main(["--channels", "1", "--samples", "16", "--output", str(output), "--no-manifest"])

    assert output.exists()
    assert not output.with_suffix(".json").exists()
