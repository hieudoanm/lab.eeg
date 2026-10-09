"""Deterministic synthetic EEG recording generation.

Fixtures are generated from a fixed seed so identical inputs always produce
identical output (docs/10-testing-strategy.md §7). Signals are synthetic and
carry no clinical meaning; they exist to exercise upload, validation, and
analysis without committing real participant recordings.

Orientation is ``(channels, samples)`` everywhere, matching the recording
contract in ``app.scientific.recording``.
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Literal

import numpy as np

Pattern = Literal["sinusoid", "noise", "constant", "mixed", "nonfinite"]

PATTERNS: tuple[Pattern, ...] = ("sinusoid", "noise", "constant", "mixed", "nonfinite")

_DEFAULT_SAMPLING_FREQUENCY_HZ = 256.0
_DEFAULT_AMPLITUDE = 50.0
_DEFAULT_FILENAME = "recording.npy"


@dataclass(frozen=True, slots=True)
class SyntheticRecordingConfig:
    """Parameters for a reproducible synthetic recording."""

    channel_count: int = 8
    sample_count: int = 2000
    sampling_frequency_hz: float = _DEFAULT_SAMPLING_FREQUENCY_HZ
    seed: int = 0
    pattern: Pattern = "mixed"
    amplitude: float = _DEFAULT_AMPLITUDE

    def __post_init__(self) -> None:
        if self.channel_count < 1:
            raise ValueError("channel_count must be at least 1")
        if self.sample_count < 1:
            raise ValueError("sample_count must be at least 1")
        if not np.isfinite(self.sampling_frequency_hz) or self.sampling_frequency_hz <= 0:
            raise ValueError("sampling_frequency_hz must be finite and greater than zero")
        if not np.isfinite(self.amplitude) or self.amplitude <= 0:
            raise ValueError("amplitude must be finite and greater than zero")


@dataclass(frozen=True, slots=True)
class SyntheticManifest:
    """Recording metadata accompanying a generated array."""

    sampling_frequency_hz: float
    channel_names: list[str]
    channel_units: list[str]
    pattern: Pattern
    seed: int

    def to_upload_dict(self) -> dict[str, Any]:
        """Return the subset the upload manifest expects."""
        return {
            "sampling_frequency_hz": self.sampling_frequency_hz,
            "channel_names": list(self.channel_names),
            "channel_units": list(self.channel_units),
        }

    def to_json_dict(self) -> dict[str, Any]:
        """Return the full manifest, including generation provenance."""
        return {
            **self.to_upload_dict(),
            "pattern": self.pattern,
            "seed": self.seed,
        }


def channel_names(channel_count: int) -> list[str]:
    """Return stable generic channel labels, e.g. ``EEG 001``."""
    return [f"EEG {index + 1:03d}" for index in range(channel_count)]


@dataclass(frozen=True, slots=True)
class SyntheticRecording:
    """A generated array and the manifest describing it."""

    array: np.ndarray
    manifest: SyntheticManifest


def _channel_signal(
    rng: np.random.Generator,
    config: SyntheticRecordingConfig,
    channel_index: int,
) -> np.ndarray:
    sample_count = config.sample_count
    if config.pattern == "constant":
        return np.full(sample_count, float(channel_index + 1), dtype=np.float64)

    if config.pattern == "noise":
        return rng.normal(loc=0.0, scale=config.amplitude * 0.1, size=sample_count)

    time = np.arange(sample_count, dtype=np.float64) / config.sampling_frequency_hz
    frequency_hz = 1.0 + channel_index
    phase = channel_index * 0.1
    oscillation = config.amplitude * np.sin(2.0 * np.pi * frequency_hz * time + phase)

    if config.pattern == "sinusoid":
        return oscillation

    if config.pattern == "nonfinite":
        signal = oscillation.copy()
        stride = max(sample_count // 8, 1)
        signal[::stride] = np.nan
        return signal

    return oscillation + rng.normal(0.0, config.amplitude * 0.05, size=sample_count)


def generate_recording(config: SyntheticRecordingConfig) -> SyntheticRecording:
    """Generate a deterministic ``(channels, samples)`` float32 recording."""
    rng = np.random.default_rng(config.seed)
    channels = [_channel_signal(rng, config, index) for index in range(config.channel_count)]
    array = np.stack(channels).astype(np.float32)
    manifest = SyntheticManifest(
        sampling_frequency_hz=config.sampling_frequency_hz,
        channel_names=channel_names(config.channel_count),
        channel_units=["uV"] * config.channel_count,
        pattern=config.pattern,
        seed=config.seed,
    )
    return SyntheticRecording(array=array, manifest=manifest)


def write_recording(
    recording: SyntheticRecording,
    output: Path,
    *,
    manifest_path: Path | None = None,
) -> None:
    """Write ``recording.npy`` and, when requested, its JSON manifest."""
    output.parent.mkdir(parents=True, exist_ok=True)
    np.save(output, recording.array)
    if manifest_path is not None:
        manifest_path.parent.mkdir(parents=True, exist_ok=True)
        manifest_path.write_text(
            json.dumps(recording.manifest.to_json_dict(), indent=2) + "\n",
            encoding="utf-8",
        )


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Generate a deterministic synthetic EEG recording (.npy).",
    )
    parser.add_argument("--channels", type=int, default=8, help="channel count (default: 8)")
    parser.add_argument(
        "--samples", type=int, default=2000, help="samples per channel (default: 2000)"
    )
    parser.add_argument(
        "--sampling-frequency",
        type=float,
        default=_DEFAULT_SAMPLING_FREQUENCY_HZ,
        help="sampling frequency in Hz (default: 256)",
    )
    parser.add_argument("--seed", type=int, default=0, help="random seed (default: 0)")
    parser.add_argument(
        "--pattern",
        choices=PATTERNS,
        default="mixed",
        help="signal shape to generate (default: mixed)",
    )
    parser.add_argument(
        "--amplitude", type=float, default=_DEFAULT_AMPLITUDE, help="signal amplitude"
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(_DEFAULT_FILENAME),
        help=f"output .npy path (default: {_DEFAULT_FILENAME})",
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        default=None,
        help="manifest JSON path (default: alongside output with .json suffix)",
    )
    parser.add_argument(
        "--no-manifest",
        action="store_true",
        help="do not write a manifest JSON file",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    """Command-line entry point for generating a synthetic recording."""
    args = _build_parser().parse_args(argv)
    config = SyntheticRecordingConfig(
        channel_count=args.channels,
        sample_count=args.samples,
        sampling_frequency_hz=args.sampling_frequency,
        seed=args.seed,
        pattern=args.pattern,
        amplitude=args.amplitude,
    )
    recording = generate_recording(config)
    manifest_path = (
        None if args.no_manifest else (args.manifest or args.output.with_suffix(".json"))
    )
    write_recording(recording, args.output, manifest_path=manifest_path)
    print(
        f"Wrote {config.channel_count} channels x {config.sample_count} samples "
        f"({args.pattern}) to {args.output}"
    )
    if manifest_path is not None:
        print(f"Wrote manifest to {manifest_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())


__all__ = [
    "PATTERNS",
    "Pattern",
    "SyntheticManifest",
    "SyntheticRecording",
    "SyntheticRecordingConfig",
    "channel_names",
    "generate_recording",
    "main",
    "write_recording",
]
