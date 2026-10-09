"""Local filesystem object-storage adapter.

Used for local development and CI. Objects are stored under a configured root
directory. Writes are atomic (temp file + rename) so a crashed upload never
leaves a half-written object behind, and keys are validated to stay inside the
root as defence in depth against accidental path traversal.
"""

from __future__ import annotations

import asyncio
import hashlib
import os
from collections.abc import AsyncIterator
from pathlib import Path

from app.storage.base import Storage

_READ_CHUNK_BYTES = 1024 * 1024


class LocalStorage:
    """A filesystem-backed object store rooted at ``root``."""

    def __init__(self, root: Path) -> None:
        self._root = root.resolve()
        self._root.mkdir(parents=True, exist_ok=True)

    def _resolve(self, key: str) -> Path:
        if not key or "\x00" in key:
            raise ValueError("object key must not be empty or contain NUL bytes")
        candidate = Path(key)
        if candidate.is_absolute():
            raise ValueError("object key must be relative")
        path = self._root.joinpath(*candidate.parts)
        resolved = path.resolve()
        if not resolved.is_relative_to(self._root):
            raise ValueError("object key escapes the storage root")
        return resolved

    async def write_bytes(self, key: str, data: bytes) -> None:
        await asyncio.to_thread(self._write_bytes, key, data)

    async def stream_write(self, key: str, chunks: AsyncIterator[bytes]) -> tuple[int, str]:
        digest = hashlib.sha256()
        written = 0
        target = self._resolve(key)
        target.parent.mkdir(parents=True, exist_ok=True)
        temp = target.with_name(f".{target.name}.tmp")
        try:
            async for chunk in chunks:
                data = bytes(chunk)
                written += len(data)
                digest.update(data)
                await asyncio.to_thread(self._append_temp, temp, data)
            await asyncio.to_thread(self._commit_temp, temp, target)
        except BaseException:
            await asyncio.to_thread(self._remove_if_exists, temp)
            raise
        return written, digest.hexdigest()

    async def exists(self, key: str) -> bool:
        return await asyncio.to_thread(self._resolve(key).exists)

    async def size(self, key: str) -> int:
        path = self._resolve(key)
        return await asyncio.to_thread(os.path.getsize, path)

    async def read_bytes(self, key: str, *, max_bytes: int | None = None) -> bytes:
        return await asyncio.to_thread(self._read_bytes, key, max_bytes)

    async def hash_sha256(self, key: str) -> str:
        return await asyncio.to_thread(self._hash_sha256, key)

    async def delete(self, key: str) -> None:
        def _remove() -> None:
            path = self._resolve(key)
            if path.exists():
                path.unlink()

        await asyncio.to_thread(_remove)

    def local_path(self, key: str) -> Path | None:
        return self._resolve(key)

    # Synchronous helpers (run inside the event-loop thread pool) -------------

    def _write_bytes(self, key: str, data: bytes) -> None:
        target = self._resolve(key)
        target.parent.mkdir(parents=True, exist_ok=True)
        temp = target.with_name(f".{target.name}.tmp")
        with temp.open("wb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        self._commit_temp(temp, target)

    def _read_bytes(self, key: str, max_bytes: int | None) -> bytes:
        path = self._resolve(key)
        with path.open("rb") as handle:
            if max_bytes is not None:
                return handle.read(max_bytes)
            return handle.read()

    def _hash_sha256(self, key: str) -> str:
        digest = hashlib.sha256()
        path = self._resolve(key)
        with path.open("rb") as handle:
            while chunk := handle.read(_READ_CHUNK_BYTES):
                digest.update(chunk)
        return digest.hexdigest()

    def _append_temp(self, temp: Path, data: bytes) -> None:
        with temp.open("ab") as handle:
            handle.write(data)

    def _commit_temp(self, temp: Path, target: Path) -> None:
        temp.replace(target)

    def _remove_if_exists(self, path: Path) -> None:
        path.unlink(missing_ok=True)

    def __repr__(self) -> str:
        return f"LocalStorage(root={str(self._root)!r})"


__all__ = ["LocalStorage", "Storage"]
