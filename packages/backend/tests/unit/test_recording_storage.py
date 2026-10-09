"""Unit tests for bounded recording access."""

from __future__ import annotations

from pathlib import Path

import numpy as np
from app.storage.local import LocalStorage
from app.storage.recording import open_recording


class _RemoteStorage(LocalStorage):
    """A local store that reports no direct path, forcing a materialised read."""

    def local_path(self, key: str) -> Path | None:
        return None


async def test_local_recording_is_used_in_place(tmp_path: Path) -> None:
    storage = LocalStorage(tmp_path)
    await storage.write_bytes("recording.npy", b"payload")

    async with open_recording(storage, "recording.npy") as path:
        assert path == storage.local_path("recording.npy")
        assert path.read_bytes() == b"payload"

    assert Path(path).exists()


async def test_remote_recording_is_materialised_then_removed(tmp_path: Path) -> None:
    storage = _RemoteStorage(tmp_path)
    await storage.write_bytes("recording.npy", _npy_bytes())

    async with open_recording(storage, "recording.npy", max_bytes=1024) as path:
        assert path.exists()
        materialised = path

    assert not materialised.exists()


def _npy_bytes() -> bytes:
    import io

    buffer = io.BytesIO()
    np.save(buffer, np.zeros((1, 4), dtype=np.float32), allow_pickle=False)
    return buffer.getvalue()
