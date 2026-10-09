"""FastAPI application factory."""

from __future__ import annotations

import logging
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.api.errors import install_error_handlers
from app.api.middleware import RequestContextMiddleware
from app.api.routes import api_v1_router, health_router
from app.core.config import Settings, get_settings
from app.core.logging import configure_logging
from app.core.metrics import install_metrics
from app.db.pool import Database
from app.storage import build_storage

logger = logging.getLogger("eeg-lab.app")

API_PREFIX = "/api/v1"


def create_app(settings: Settings | None = None) -> FastAPI:
    """Build the FastAPI application."""
    resolved = settings or get_settings()
    configure_logging(resolved.log_level, resolved.log_format)
    database = Database(resolved)
    storage = build_storage(resolved)

    @asynccontextmanager
    async def lifespan(app: FastAPI) -> AsyncIterator[None]:
        await database.connect()
        logger.info(
            "api started",
            extra={"environment": resolved.app_env, "version": resolved.app_version},
        )
        yield
        await database.close()
        logger.info("api stopped")

    app = FastAPI(
        title="EEG Lab API",
        version=resolved.app_version,
        description=(
            "REST API for organising neuroscience studies, EEG recordings, and "
            "reproducible descriptive analyses. Research software demonstration only."
        ),
        lifespan=lifespan,
        docs_url=None if resolved.is_production else "/docs",
        redoc_url=None if resolved.is_production else "/redoc",
        openapi_url=f"{API_PREFIX}/openapi.json",
    )

    app.state.settings = resolved
    app.state.database = database
    app.state.storage = storage

    app.add_middleware(RequestContextMiddleware, settings=resolved)
    app.add_middleware(
        CORSMiddleware,
        allow_origins=resolved.api_cors_origins,
        allow_credentials=True,
        allow_methods=["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"],
        allow_headers=["Authorization", "Content-Type", "Idempotency-Key", "X-Request-Id"],
        expose_headers=["X-Request-Id"],
    )

    install_error_handlers(app)
    install_metrics(app)
    app.include_router(health_router)
    app.include_router(api_v1_router)

    return app


app = create_app()
