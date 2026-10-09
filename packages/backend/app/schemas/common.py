"""Shared Pydantic building blocks for request and response models."""

from __future__ import annotations

from datetime import UTC, datetime
from typing import Annotated

from pydantic import BaseModel, ConfigDict, PlainSerializer


def format_utc(value: datetime) -> str:
    """Serialise timestamps as ISO 8601 UTC with a trailing ``Z``."""
    return value.astimezone(UTC).isoformat().replace("+00:00", "Z")


UtcTimestamp = Annotated[datetime, PlainSerializer(format_utc, return_type=str)]


class RequestModel(BaseModel):
    """Base for request bodies: unknown fields are rejected."""

    model_config = ConfigDict(extra="forbid")
