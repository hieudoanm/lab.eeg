"""Liveness and readiness endpoints."""

from __future__ import annotations

from fastapi import APIRouter, Depends, Response, status

from app.api.dependencies import get_database, get_settings
from app.core.config import Settings
from app.db.pool import Database
from app.schemas.health import HealthLiveResponse, HealthReadyResponse

router = APIRouter(tags=["health"])


@router.get(
    "/health/live",
    response_model=HealthLiveResponse,
    summary="Liveness probe",
    description="Reports whether the API process is alive. Never checks dependencies.",
)
async def live(settings: Settings = Depends(get_settings)) -> HealthLiveResponse:
    return HealthLiveResponse(
        service=settings.app_name,
        version=settings.app_version,
        environment=settings.app_env,
    )


@router.get(
    "/health/ready",
    response_model=HealthReadyResponse,
    summary="Readiness probe",
    description="Reports whether required dependencies are available. Exposes no secrets.",
)
async def ready(
    response: Response,
    database: Database = Depends(get_database),
    settings: Settings = Depends(get_settings),
) -> HealthReadyResponse:
    database_ok = await database.ping()
    checks = {"database": "ok" if database_ok else "unavailable"}
    is_ready = database_ok
    if not is_ready:
        response.status_code = status.HTTP_503_SERVICE_UNAVAILABLE
    return HealthReadyResponse(
        status="ready" if is_ready else "unavailable",
        service=settings.app_name,
        version=settings.app_version,
        checks=checks,
    )


@router.get(
    "/healthz",
    include_in_schema=False,
    response_model=HealthLiveResponse,
)
async def healthz_alias(settings: Settings = Depends(get_settings)) -> HealthLiveResponse:
    return await live(settings=settings)


@router.get(
    "/readyz",
    include_in_schema=False,
    response_model=HealthReadyResponse,
)
async def readyz_alias(
    response: Response,
    database: Database = Depends(get_database),
    settings: Settings = Depends(get_settings),
) -> HealthReadyResponse:
    return await ready(response=response, database=database, settings=settings)
