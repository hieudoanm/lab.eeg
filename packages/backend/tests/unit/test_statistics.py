"""Unit tests for deterministic descriptive statistics."""

from __future__ import annotations

import numpy as np
import pytest
from app.scientific import StatisticsInvalidError, compute_descriptive_statistics


def test_known_values_match_independent_calculation() -> None:
    array = np.array([[1.0, 2.0, 3.0, 4.0]], dtype=np.float64)

    statistics = compute_descriptive_statistics(array, channel_names=["F3"])

    assert len(statistics) == 1
    channel = statistics[0]
    assert channel.channel_name == "F3"
    assert channel.channel_index == 0
    assert channel.mean == pytest.approx(2.5)
    assert channel.standard_deviation == pytest.approx(np.std(array))
    assert channel.minimum == pytest.approx(1.0)
    assert channel.maximum == pytest.approx(4.0)
    assert channel.peak_to_peak == pytest.approx(3.0)
    assert channel.valid_sample_count == 4


def test_standard_deviation_uses_population_convention() -> None:
    array = np.array([[1.0, 2.0, 3.0]], dtype=np.float64)

    channel = compute_descriptive_statistics(array, channel_names=["Cz"])[0]

    assert channel.standard_deviation == pytest.approx(np.std(array, ddof=0))
    assert channel.standard_deviation != pytest.approx(np.std(array, ddof=1))


def test_non_finite_samples_are_excluded_and_counted() -> None:
    array = np.array([[1.0, np.nan, 3.0, np.inf, -np.inf]], dtype=np.float64)

    channel = compute_descriptive_statistics(array, channel_names=["F4"])[0]

    assert channel.valid_sample_count == 2
    assert channel.mean == pytest.approx(2.0)
    assert channel.standard_deviation == pytest.approx(1.0)
    assert channel.minimum == pytest.approx(1.0)
    assert channel.maximum == pytest.approx(3.0)
    assert channel.peak_to_peak == pytest.approx(2.0)


def test_channel_with_no_valid_samples_is_an_error() -> None:
    array = np.array([[np.nan, np.nan]], dtype=np.float64)

    with pytest.raises(StatisticsInvalidError, match="no valid samples"):
        compute_descriptive_statistics(array, channel_names=["T7"])


def test_disabled_metrics_are_omitted() -> None:
    array = np.array([[1.0, 2.0, 3.0]], dtype=np.float64)

    channel = compute_descriptive_statistics(
        array,
        channel_names=["P3"],
        include_mean=False,
        include_standard_deviation=False,
        include_min_max=False,
        include_peak_to_peak=False,
    )[0]

    assert channel.mean is None
    assert channel.standard_deviation is None
    assert channel.minimum is None
    assert channel.maximum is None
    assert channel.peak_to_peak is None
    assert channel.valid_sample_count == 3


def test_multi_channel_orientation_is_channels_by_samples() -> None:
    array = np.array([[1.0, 2.0], [10.0, 20.0]], dtype=np.float64)

    statistics = compute_descriptive_statistics(array, channel_names=["A", "B"])

    assert [channel.channel_name for channel in statistics] == ["A", "B"]
    assert statistics[0].mean == pytest.approx(1.5)
    assert statistics[1].mean == pytest.approx(15.0)


def test_units_are_reported_when_known() -> None:
    array = np.zeros((2, 3), dtype=np.float64)

    statistics = compute_descriptive_statistics(
        array,
        channel_names=["A", "B"],
        channel_units=["uV", "uV"],
    )

    assert [channel.unit for channel in statistics] == ["uV", "uV"]


def test_channel_name_mismatch_is_rejected() -> None:
    array = np.zeros((2, 3), dtype=np.float64)

    with pytest.raises(StatisticsInvalidError, match="channel metadata"):
        compute_descriptive_statistics(array, channel_names=["only-one"])


def test_channel_unit_mismatch_is_rejected() -> None:
    array = np.zeros((2, 3), dtype=np.float64)

    with pytest.raises(StatisticsInvalidError, match="unit metadata"):
        compute_descriptive_statistics(
            array,
            channel_names=["A", "B"],
            channel_units=["uV"],
        )


def test_non_two_dimensional_array_is_rejected() -> None:
    array = np.zeros((3,), dtype=np.float64)

    with pytest.raises(StatisticsInvalidError, match="two-dimensional"):
        compute_descriptive_statistics(array, channel_names=["A"])


def test_integer_dtype_is_supported() -> None:
    array = np.array([[1, 2, 3, 4]], dtype=np.int16)

    channel = compute_descriptive_statistics(array, channel_names=["F3"])[0]

    assert channel.mean == pytest.approx(2.5)
    assert channel.valid_sample_count == 4
