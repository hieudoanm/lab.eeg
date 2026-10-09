"""API route registration."""

from __future__ import annotations

from fastapi import APIRouter

from app.api.routes.analyses import router as analyses_router
from app.api.routes.auth import router as auth_router
from app.api.routes.datasets import router as datasets_router
from app.api.routes.health import router as health_router
from app.api.routes.organisations import router as organisations_router
from app.api.routes.signals import router as signals_router
from app.api.routes.studies import router as studies_router
from app.api.routes.summary import router as summary_router
from app.api.routes.uploads import router as uploads_router
from app.api.routes.users import router as users_router

api_v1_router = APIRouter(prefix="/api/v1")
api_v1_router.include_router(auth_router)
api_v1_router.include_router(users_router)
api_v1_router.include_router(organisations_router)
api_v1_router.include_router(summary_router)
api_v1_router.include_router(studies_router)
api_v1_router.include_router(datasets_router)
api_v1_router.include_router(uploads_router)
api_v1_router.include_router(analyses_router)
api_v1_router.include_router(signals_router)

__all__ = ["api_v1_router", "health_router"]
