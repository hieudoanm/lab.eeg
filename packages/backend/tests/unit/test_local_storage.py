"""Unit tests for the local filesystem object-storage adapter."""

from __future__ import annotations

import hashlib
from collections.abc import AsyncIterator
from pathlib import Path

import pytest
from app.storage import build_storage
from app.storage.local import LocalStorage

from tests.conftest import make_settings


@pytest.fixture
def storage(tmp_path: Path) -> LocalStorage:
    return LocalStorage(tmp_path)


async def _stream(*chunks: bytes) -> AsyncIterator[bytes]:
    for chunk in chunks:
        yield chunk


def _object_absent(storage: LocalStorage, key: str) -> bool:
    path = storage.local_path(key)
    assert path is not None
    return not path.exists()


def test_constructor_creates_the_root(tmp_path: Path) -> None:
    root = tmp_path / "nested" / "objects"
    instance = LocalStorage(root)
    assert root.is_dir()
    assert repr(instance) == f"LocalStorage(root={str(root.resolve())!r})"


def test_resolve_rejects_invalid_keys(storage: LocalStorage, tmp_path: Path) -> None:
    with pytest.raises(ValueError):
        storage.local_path("")
    with pytest.raises(ValueError):
        storage.local_path("bad\x00key")
    with pytest.raises(ValueError):
        storage.local_path(str(tmp_path / "absolute"))
    with pytest.raises(ValueError):
        storage.local_path("../escape")


async def test_write_read_exists_size_and_hash(storage: LocalStorage) -> None:
    payload = b"recorded samples"
    await storage.write_bytes("org/dataset/recording.npy", payload)

    assert await storage.exists("org/dataset/recording.npy") is True
    assert await storage.exists("org/dataset/missing.npy") is False
    assert await storage.size("org/dataset/recording.npy") == len(payload)
    assert await storage.read_bytes("org/dataset/recording.npy") == payload
    assert (
        await storage.hash_sha256("org/dataset/recording.npy")
        == hashlib.sha256(payload).hexdigest()
    )
    assert storage.local_path("org/dataset/recording.npy") is not None


async def test_read_bytes_honours_max_bytes(storage: LocalStorage) -> None:
    await storage.write_bytes("clip.bin", b"0123456789")

    assert await storage.read_bytes("clip.bin", max_bytes=4) == b"0123"


async def test_stream_write_hashes_and_commits(storage: LocalStorage) -> None:
    written, digest = await storage.stream_write("stream/recording.npy", _stream(b"ab", b"cd"))

    assert written == 4
    assert digest == hashlib.sha256(b"abcd").hexdigest()
    assert await storage.read_bytes("stream/recording.npy") == b"abcd"
    assert _object_absent(storage, "stream/.recording.npy.tmp")


async def test_stream_write_cleans_up_on_failure(storage: LocalStorage) -> None:
    async def failing() -> AsyncIterator[bytes]:
        yield b"partial"
        raise RuntimeError("stream interrupted")

    with pytest.raises(RuntimeError):
        await storage.stream_write("stream/broken.npy", failing())

    assert _object_absent(storage, "stream/broken.npy")
    assert _object_absent(storage, "stream/.broken.npy.tmp")


async def test_delete_removes_existing_and_ignores_missing(storage: LocalStorage) -> None:
    await storage.write_bytes("to-delete.bin", b"data")

    await storage.delete("to-delete.bin")
    assert await storage.exists("to-delete.bin") is False
    # Deleting a missing object is a no-op.
    await storage.delete("to-delete.bin")


def test_build_storage_selects_local() -> None:
    storage = build_storage(make_settings())
    assert isinstance(storage, LocalStorage)


def test_build_storage_rejects_unknown_backend() -> None:
    with pytest.raises(ValueError):
        build_storage(make_settings(storage_backend="gcs"))
