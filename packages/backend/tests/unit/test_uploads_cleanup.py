"""Unit tests for abandoned-upload cleanup argument handling."""

from __future__ import annotations

from typing import cast

import pytest
from app.db.pool import Database
from app.services.uploads import cleanup_abandoned_uploads
from app.storage.base import Storage


async def test_cleanup_rejects_non_positive_batch_size() -> None:
    with pytest.raises(ValueError, match="batch_size"):
        await cleanup_abandoned_uploads(
            cast(Database, object()),
            cast(Storage, object()),
            batch_size=0,
        )


async def test_cleanup_rejects_non_positive_max_batches() -> None:
    with pytest.raises(ValueError, match="max_batches"):
        await cleanup_abandoned_uploads(
            cast(Database, object()),
            cast(Storage, object()),
            max_batches=0,
        )
