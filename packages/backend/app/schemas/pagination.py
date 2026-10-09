"""Pagination response schema."""

from __future__ import annotations

from collections.abc import Sequence
from typing import TypeVar

from pydantic import BaseModel, Field

T = TypeVar("T")


class PageResponse[T](BaseModel):
    """Generic cursor-paginated response."""

    items: Sequence[T]
    next_cursor: str | None = Field(default=None)
    has_more: bool = Field(default=False)
