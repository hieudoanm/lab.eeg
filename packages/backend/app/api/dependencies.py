"""Shared FastAPI dependencies."""

from __future__ import annotations

from typing import cast
from uuid import UUID

from fastapi import Depends, Request

from app.core.config import Settings
from app.core.exceptions import ForbiddenError, UnauthorizedError
from app.core.security import decode_access_token
from app.db.pool import Database
from app.models.identity import OrganisationContext, User
from app.services import identity, tenancy
from app.storage.base import Storage


def get_settings(request: Request) -> Settings:
    """Return the application settings bound to the running process."""
    return cast(Settings, request.app.state.settings)


def get_database(request: Request) -> Database:
    """Return the database pool bound to the running process."""
    return cast(Database, request.app.state.database)


def get_storage(request: Request) -> Storage:
    """Return the object-storage adapter bound to the running process."""
    return cast(Storage, request.app.state.storage)


def get_request_id(request: Request) -> str | None:
    """Return the request correlation identifier set by the middleware."""
    value = getattr(request.state, "request_id", None)
    return value if isinstance(value, str) else None


def _extract_access_token(request: Request, settings: Settings) -> str:
    """Read the access token from the Authorization header or session cookie."""
    header = request.headers.get("Authorization")
    if header:
        scheme, _, credentials = header.partition(" ")
        if scheme.lower() != "bearer" or not credentials.strip():
            raise UnauthorizedError("The access token is invalid.", code="invalid_token")
        return credentials.strip()
    cookie = request.cookies.get(settings.auth_cookie_name)
    if cookie:
        return cookie
    raise UnauthorizedError("Authentication is required.", code="authentication_required")


async def get_current_user(
    request: Request,
    settings: Settings = Depends(get_settings),
    database: Database = Depends(get_database),
) -> User:
    """Authenticate the request and return (provisioning if needed) the user."""
    claims = decode_access_token(settings, _extract_access_token(request, settings))
    user = await identity.provision_user(
        database,
        identity_provider=claims["idp"],
        external_subject=claims["sub"],
        display_name=claims.get("name"),
        email=claims.get("email"),
    )
    if not user.is_active:
        raise ForbiddenError("This account is disabled.", code="account_disabled")
    return user


async def get_organisation_context(
    organisation_id: UUID,
    user: User = Depends(get_current_user),
    database: Database = Depends(get_database),
) -> OrganisationContext:
    """Resolve the verified tenant context for the path organisation."""
    return await tenancy.load_organisation_context(
        database, user=user, organisation_id=organisation_id
    )
