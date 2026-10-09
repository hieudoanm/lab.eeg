"""Opaque cursor pagination helpers.

Collections that may grow use keyset pagination over a deterministic
``(created_at DESC, id DESC)`` ordering. A cursor is an opaque, URL-safe token
that encodes the last row's ordering key. It carries no authorisation meaning:
the query always applies the verified organisation scope first, and invalid or
foreign cursors simply yield an empty page.
"""

from __future__ import annotations

import base64
import binascii
import json
from dataclasses import dataclass
from datetime import UTC, datetime
from uuid import UUID

from app.core.exceptions import UnprocessableEntityError


@dataclass(frozen=True, slots=True)
class Cursor:
    """The ordering key of the last item on the previous page."""

    created_at: datetime
    id: UUID


def encode_cursor(*, created_at: datetime, id: UUID) -> str:
    """Encode an ordering key into an opaque, URL-safe cursor."""
    payload = {"t": created_at.astimezone(UTC).isoformat(), "i": str(id)}
    raw = json.dumps(payload, separators=(",", ":"), sort_keys=True).encode("utf-8")
    return base64.urlsafe_b64encode(raw).decode("ascii").rstrip("=")


def decode_cursor(value: str) -> Cursor:
    """Decode a cursor, raising a validation error when it is malformed."""
    try:
        padding = "=" * (-len(value) % 4)
        raw = base64.urlsafe_b64decode(value + padding)
        payload = json.loads(raw)
        created_at = datetime.fromisoformat(payload["t"])
        if created_at.tzinfo is None:
            raise ValueError("cursor timestamp must be timezone-aware")
        identifier = UUID(payload["i"])
    except (binascii.Error, ValueError, KeyError, TypeError, json.JSONDecodeError) as exc:
        raise UnprocessableEntityError(
            "The pagination cursor is invalid.",
            code="invalid_cursor",
        ) from exc
    return Cursor(created_at=created_at.astimezone(UTC), id=identifier)
