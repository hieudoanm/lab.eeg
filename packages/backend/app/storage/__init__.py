"""Object-storage adapter selection."""

from __future__ import annotations

from pathlib import Path

from app.core.config import Settings
from app.storage.base import Storage
from app.storage.local import LocalStorage


def build_storage(settings: Settings) -> Storage:
    """Return the storage adapter configured for the running process."""
    if settings.storage_backend == "local":
        return LocalStorage(Path(settings.local_storage_path))
    raise ValueError(f"storage backend {settings.storage_backend!r} is not implemented yet")


__all__ = ["LocalStorage", "Storage", "build_storage"]
