"""Asynchronous PostgreSQL connection pool.

Repositories receive an open connection and execute parameterised SQL. The pool
owns connection lifecycle; services own transaction boundaries. The pool opens
lazily so that the API process can start while PostgreSQL is unavailable and
report readiness through ``/health/ready`` instead of crashing.
"""

from __future__ import annotations

import asyncio
import logging
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from typing import Any

from psycopg import AsyncConnection
from psycopg.rows import dict_row
from psycopg_pool import AsyncConnectionPool

from app.core.config import Settings
from app.core.exceptions import ServiceUnavailableError

logger = logging.getLogger("eeg-lab.db")

DictConnection = AsyncConnection[dict[str, Any]]


class Database:
    """Lazy, self-healing wrapper around an asynchronous psycopg pool."""

    def __init__(self, settings: Settings) -> None:
        self._settings = settings
        self._pool: AsyncConnectionPool | None = None
        self._lock = asyncio.Lock()

    @property
    def is_ready(self) -> bool:
        return self._pool is not None and not self._pool.closed

    async def connect(self) -> None:
        """Open the pool, tolerating a database that is not yet reachable."""
        try:
            await self._open_pool()
        except Exception:
            logger.warning(
                "database unavailable at startup; readiness will report unavailable",
                exc_info=True,
            )

    async def close(self) -> None:
        pool, self._pool = self._pool, None
        if pool is not None and not pool.closed:
            await pool.close()

    async def _open_pool(self) -> AsyncConnectionPool:
        if self._pool is not None and not self._pool.closed:
            return self._pool
        async with self._lock:
            if self._pool is not None and not self._pool.closed:
                return self._pool
            pool = AsyncConnectionPool(
                conninfo=self._settings.database_url,
                min_size=self._settings.database_pool_min_size,
                max_size=self._settings.database_pool_max_size,
                open=False,
                kwargs={
                    "row_factory": dict_row,
                    "connect_timeout": int(self._settings.database_connect_timeout_seconds),
                    "options": (
                        f"-c statement_timeout={self._settings.database_statement_timeout_ms}"
                    ),
                    "application_name": self._settings.app_name,
                },
            )
            try:
                await pool.open(
                    wait=True,
                    timeout=self._settings.database_connect_timeout_seconds,
                )
            except BaseException:
                await pool.close()
                raise
            self._pool = pool
            logger.info(
                "database pool opened",
                extra={
                    "min_size": self._settings.database_pool_min_size,
                    "max_size": self._settings.database_pool_max_size,
                },
            )
            return pool

    async def _require_pool(self) -> AsyncConnectionPool:
        if self.is_ready:
            assert self._pool is not None
            return self._pool
        try:
            return await self._open_pool()
        except Exception as exc:
            raise ServiceUnavailableError("The database is temporarily unavailable.") from exc

    @asynccontextmanager
    async def connection(self) -> AsyncIterator[DictConnection]:
        """Acquire a connection without an implicit transaction."""
        pool = await self._require_pool()
        async with pool.connection() as connection:
            yield connection  # type: ignore[misc]

    @asynccontextmanager
    async def transaction(self) -> AsyncIterator[DictConnection]:
        """Acquire a connection wrapped in an explicit transaction."""
        pool = await self._require_pool()
        async with pool.connection() as connection, connection.transaction():
            yield connection  # type: ignore[misc]

    async def ping(self) -> bool:
        """Return True when the database answers a trivial query."""
        try:
            async with self.connection() as connection:
                cursor = await connection.execute("SELECT 1 AS ok")
                row = await cursor.fetchone()
                return bool(row and row["ok"] == 1)
        except ServiceUnavailableError:
            return False
        except Exception:
            logger.warning("database readiness check failed", exc_info=True)
            return False
