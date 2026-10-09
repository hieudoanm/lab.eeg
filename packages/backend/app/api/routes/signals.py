"""Signal display endpoint.

The endpoint returns a bounded, decimated window of a ready dataset's recording
for charting. It is read-only: it never feeds analysis and never mutates the
stored recording. Windows are described in seconds and channel indices are
supplied as a comma-separated list, for example
``?start_seconds=1&end_seconds=2&channels=0,1``.
"""

from __future__ import annotations

from uuid import UUID

from fastapi import APIRouter, Depends, Query

from app.api.dependencies import (
    get_database,
    get_organisation_context,
    get_settings,
    get_storage,
)
from app.core.config import Settings
from app.core.exceptions import UnprocessableEntityError
from app.db.pool import Database
from app.models.identity import OrganisationContext
from app.schemas.signal import SignalChannelRead, SignalWindowRead
from app.services import signals as signals_service
from app.storage.base import Storage

router = APIRouter(tags=["signals"])


def _parse_channels(raw: str | None) -> list[int] | None:
    """Parse a comma-separated channel selection, rejecting non-integers."""
    if raw is None or not raw.strip():
        return None
    parts = [part.strip() for part in raw.split(",") if part.strip()]
    try:
        return [int(part) for part in parts]
    except ValueError as exc:
        raise UnprocessableEntityError(
            "Channel indices must be integers.",
            code="invalid_signal_window",
        ) from exc


@router.get(
    "/organisations/{organisation_id}/datasets/{dataset_id}/signals",
    response_model=SignalWindowRead,
    summary="Read a bounded window of a dataset's recording for display",
)
async def read_signal_window(
    dataset_id: UUID,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
    storage: Storage = Depends(get_storage),
    settings: Settings = Depends(get_settings),
    start_seconds: float = Query(default=0.0, ge=0.0),
    end_seconds: float = Query(..., gt=0.0),
    channels: str | None = Query(default=None),
) -> SignalWindowRead:
    window = await signals_service.read_signal_window(
        database,
        storage,
        settings,
        context=context,
        dataset_id=dataset_id,
        start_seconds=start_seconds,
        end_seconds=end_seconds,
        channels=_parse_channels(channels),
    )
    return SignalWindowRead(
        dataset_id=dataset_id,
        sampling_frequency_hz=window.metadata.sampling_frequency_hz,
        channel_count=window.metadata.channel_count,
        sample_count=window.metadata.sample_count,
        duration_seconds=window.metadata.duration_seconds,
        start_seconds=window.start_seconds,
        end_seconds=window.end_seconds,
        sample_start=window.sample_start,
        sample_end=window.sample_end,
        channel_indices=[channel.channel_index for channel in window.channels],
        points_per_channel=window.points_per_channel,
        downsampled=window.downsampled,
        downsample_factor=window.downsample_factor,
        time_seconds=window.time_seconds,
        channels=[
            SignalChannelRead(
                channel_index=channel.channel_index,
                channel_name=channel.channel_name,
                unit=channel.unit,
                samples=channel.samples,
            )
            for channel in window.channels
        ],
    )
