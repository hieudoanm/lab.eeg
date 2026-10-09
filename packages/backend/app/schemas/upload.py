"""Upload session schemas: request and response models."""

from __future__ import annotations

import re
from typing import Literal
from uuid import UUID

from pydantic import Field, field_validator

from app.models.dataset import DatasetFormat
from app.models.upload_session import UploadSessionStatus
from app.schemas.common import RequestModel, UtcTimestamp
from app.schemas.dataset import DatasetRead

_SHA256_PATTERN = re.compile(r"^[0-9a-f]{64}$")


class UploadManifest(RequestModel):
    """Metadata supplied with the recording and verified against the array.

    The ``.npy`` array embeds no sampling frequency or channel labels, so the
    manifest is the authoritative source for these values; validation rejects a
    manifest that does not match the array's first dimension.
    """

    sampling_frequency_hz: float = Field(gt=0)
    channel_names: list[str] = Field(min_length=1, max_length=256)
    channel_units: list[str] | None = Field(default=None, max_length=256)

    @field_validator("sampling_frequency_hz")
    @classmethod
    def _finite_frequency(cls, value: float) -> float:
        import math

        if not math.isfinite(value) or value <= 0:
            raise ValueError("sampling frequency must be finite and greater than zero")
        return value

    @field_validator("channel_names")
    @classmethod
    def _clean_channel_names(cls, value: list[str]) -> list[str]:
        cleaned: list[str] = []
        for name in value:
            stripped = name.strip()
            if not stripped:
                raise ValueError("channel names must not be blank")
            if len(stripped) > 100:
                raise ValueError("channel names are limited to 100 characters")
            cleaned.append(stripped)
        return cleaned


class UploadSessionCreateRequest(RequestModel):
    """Create an upload session for a registered dataset."""

    file_name: str = Field(min_length=1, max_length=255)
    file_size_bytes: int = Field(ge=1)
    format: DatasetFormat = DatasetFormat.NPY
    expected_checksum_sha256: str | None = Field(default=None, min_length=64, max_length=64)
    manifest: UploadManifest

    @field_validator("expected_checksum_sha256")
    @classmethod
    def _validate_checksum(cls, value: str | None) -> str | None:
        if value is None:
            return None
        if not _SHA256_PATTERN.fullmatch(value):
            raise ValueError("expected checksum must be a 64-character SHA-256 hex digest")
        return value

    @field_validator("file_name", mode="before")
    @classmethod
    def _strip_file_name(cls, value: object) -> object:
        return value.strip() if isinstance(value, str) else value


UploadMethod = Literal["backend", "signed_url"]


class UploadSessionRead(RequestModel):
    """An upload session handed to the client.

    For the local ``backend`` method, ``upload_url`` is a relative path on the
    same API host; the caller authenticates as it does for any other request.
    """

    upload_session_id: UUID
    status: UploadSessionStatus
    upload_method: UploadMethod
    upload_url: str
    expires_at: UtcTimestamp


class UploadFinaliseResponse(RequestModel):
    """Result of a finalisation request.

    Validation runs synchronously in Phase 5 (the background worker arrives in
    Phase 6), so the dataset is returned in its final state.
    """

    upload_session_id: UUID
    upload_method: Literal["backend"] = "backend"
    dataset: DatasetRead
    status: UploadSessionStatus


__all__ = [
    "UploadFinaliseResponse",
    "UploadManifest",
    "UploadSessionCreateRequest",
    "UploadSessionRead",
]
