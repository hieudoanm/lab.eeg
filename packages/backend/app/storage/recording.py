"""Bounded access to stored recordings for read-only consumers.

Both the analysis worker and the signal display endpoint need a filesystem path
to a stored ``.npy`` object. Locally backed storage exposes one directly so
callers can memory-map it; remote storage is materialised into a temporary file
that is removed as soon as the consumer finishes. Remote reads are bounded so a
pathological upload can never be pulled into API or worker memory in full.
"""

from __future__ import annotations

from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from pathlib import Path

from app.storage.base import Storage


@asynccontextmanager
async def open_recording(
    storage: Storage,
    key: str,
    *,
    max_bytes: int | None = None,
) -> AsyncIterator[Path]:
    """Yield a readable path for ``key``, materialising remote objects.

    When the adapter is backed by a local filesystem the returned path points at
    the stored object; otherwise the object is copied into a temporary file. The
    temporary file is deleted when the context exits.
    """
    local = storage.local_path(key)
    if local is not None:
        yield local
        return

    import tempfile

    data = await storage.read_bytes(key, max_bytes=max_bytes)
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "recording.npy"
        path.write_bytes(data)
        yield path


__all__ = ["open_recording"]
