"""Object-storage adapter interface.

Large scientific recordings are never routed through JSON endpoints. They are
written to private object storage and referenced only by a server-generated key.
The storage adapter abstracts the backing store (local filesystem today, cloud
object storage later) behind a small async surface so services stay independent
of the transport.

Adapters must expose a bounded ``read_bytes`` and a streaming ``stream_write``
so that recordings can be persisted or hashed without loading the entire object
into API memory.
"""

from __future__ import annotations

import hashlib
from abc import abstractmethod
from collections.abc import AsyncIterator
from pathlib import Path
from typing import Protocol, runtime_checkable


@runtime_checkable
class Storage(Protocol):
    """Object-storage operations used by the upload and analysis services."""

    @abstractmethod
    async def write_bytes(self, key: str, data: bytes) -> None:
        """Persist a complete object. Used only for small payloads."""

    @abstractmethod
    async def stream_write(self, key: str, chunks: AsyncIterator[bytes]) -> tuple[int, str]:
        """Stream an object to storage, returning ``(size_bytes, sha256_hex)``."""

    @abstractmethod
    async def exists(self, key: str) -> bool:
        """Return whether an object exists."""

    @abstractmethod
    async def size(self, key: str) -> int:
        """Return the object size in bytes."""

    @abstractmethod
    async def read_bytes(self, key: str, *, max_bytes: int | None = None) -> bytes:
        """Read an object, bounded by ``max_bytes`` when provided."""

    @abstractmethod
    async def hash_sha256(self, key: str) -> str:
        """Return the SHA-256 hex digest of the object content."""

    @abstractmethod
    async def delete(self, key: str) -> None:
        """Remove an object if it exists. Not used for immutable recordings."""

    @abstractmethod
    def local_path(self, key: str) -> Path | None:
        """Return a local filesystem path for the key, if backed by one.

        When this returns ``None`` callers must fall back to bounded remote
        reads; when it returns a path they may use memory-mapped access.
        """


def hash_of_bytes(data: bytes) -> str:
    """Return the SHA-256 hex digest of a byte payload."""
    return hashlib.sha256(data).hexdigest()
