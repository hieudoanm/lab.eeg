"""Signal window reads for the interactive chart display.

Signal reads are strictly read-only display operations. The tenant scope is
resolved from the verified organisation context before the dataset is loaded, so
another organisation's dataset is reported as not-found. The stored recording is
opened memory-mapped and only the requested window is sliced and decimated, so a
browser never receives an unbounded payload.
"""

from __future__ import annotations

import asyncio
from uuid import UUID

from app.core.config import Settings
from app.core.exceptions import ConflictError, NotFoundError, UnprocessableEntityError
from app.db.pool import Database
from app.models.dataset import DatasetStatus
from app.models.identity import OrganisationContext
from app.repositories import datasets as datasets_repository
from app.scientific.recording import ChannelManifest, RecordingInvalidError
from app.scientific.signals import (
    SignalWindow,
    SignalWindowInvalidError,
    extract_signal_window,
)
from app.storage.base import Storage
from app.storage.recording import open_recording

#: Channel returned when the caller does not select one explicitly.
_DEFAULT_CHANNELS = (0,)


async def read_signal_window(
    database: Database,
    storage: Storage,
    settings: Settings,
    *,
    context: OrganisationContext,
    dataset_id: UUID,
    start_seconds: float,
    end_seconds: float,
    channels: list[int] | None,
) -> SignalWindow:
    """Read a bounded, decimated window of a dataset's recording."""
    async with database.connection() as connection:
        dataset = await datasets_repository.get_dataset(
            connection,
            organisation_id=context.organisation.id,
            dataset_id=dataset_id,
        )
    if dataset is None:
        raise NotFoundError("The requested dataset does not exist.")
    if dataset.status is not DatasetStatus.READY or dataset.channel_count is None:
        raise ConflictError(
            "Only a validated, ready dataset can be displayed.",
            code="dataset_not_ready",
        )
    if dataset.storage_key is None:
        raise ConflictError(
            "Only a validated, ready dataset can be displayed.",
            code="dataset_not_ready",
        )
    if not await storage.exists(dataset.storage_key):
        raise ConflictError(
            "The recording could not be found.",
            code="dataset_recording_missing",
        )

    selected = _selected_channels(channels, dataset.channel_count)
    max_points_per_channel = max(1, settings.max_signal_points_per_request // max(1, len(selected)))
    manifest = ChannelManifest(
        sampling_frequency_hz=dataset.sampling_frequency_hz,
        channel_names=dataset.channel_names or [],
        channel_units=dataset.channel_units,
    )

    async with open_recording(
        storage, dataset.storage_key, max_bytes=settings.max_upload_bytes
    ) as path:
        try:
            return await asyncio.to_thread(
                extract_signal_window,
                path,
                manifest=manifest,
                start_seconds=start_seconds,
                end_seconds=end_seconds,
                channel_indices=selected,
                max_channels=settings.max_channels,
                max_sample_count=settings.max_sample_count,
                max_window_seconds=settings.max_signal_window_seconds,
                max_points_per_channel=max_points_per_channel,
            )
        except SignalWindowInvalidError as exc:
            raise UnprocessableEntityError(str(exc), code="invalid_signal_window") from exc
        except RecordingInvalidError as exc:
            raise ConflictError(
                "The recording could not be read.",
                code="dataset_recording_invalid",
            ) from exc


def _selected_channels(channels: list[int] | None, channel_count: int) -> list[int]:
    """Return the requested channels, defaulting to the first channel."""
    if channels:
        return channels
    candidates = [index for index in _DEFAULT_CHANNELS if index < channel_count]
    if not candidates:
        raise UnprocessableEntityError(
            "A channel must be selected for this dataset.",
            code="invalid_channel_selection",
        )
    return candidates


__all__ = ["read_signal_window"]
