"""Development-only access token issuance and local sign-out."""

from __future__ import annotations

from fastapi import APIRouter, Depends, Response, status

from app.api.cookies import clear_session_cookie, set_session_cookie
from app.api.dependencies import get_database, get_settings
from app.core.config import Settings
from app.core.exceptions import NotFoundError
from app.core.security import create_access_token
from app.db.pool import Database
from app.schemas.identity import AccessTokenResponse, DevelopmentTokenRequest
from app.services import identity

router = APIRouter(tags=["authentication"])


@router.post(
    "/auth/token",
    response_model=AccessTokenResponse,
    summary="Issue a development access token",
    description=(
        "Mints a short-lived bearer token for local development and tests, and "
        "sets an HttpOnly session cookie for browser clients. The endpoint is "
        "disabled whenever AUTH_MODE is not 'development' and never serves "
        "production traffic."
    ),
)
async def issue_development_token(
    payload: DevelopmentTokenRequest,
    response: Response,
    settings: Settings = Depends(get_settings),
    database: Database = Depends(get_database),
) -> AccessTokenResponse:
    if not settings.development_auth_enabled:
        raise NotFoundError("The requested resource does not exist.")

    provider = payload.identity_provider or settings.dev_identity_provider
    subject = payload.subject or settings.dev_identity_subject
    display_name = (
        payload.display_name
        if payload.display_name is not None
        else settings.dev_identity_display_name
    )
    email = payload.email if payload.email is not None else settings.dev_identity_email

    user = await identity.provision_user(
        database,
        identity_provider=provider,
        external_subject=subject,
        display_name=display_name,
        email=email,
    )
    await identity.ensure_development_organisation(
        database, user=user, name=settings.dev_organisation_name
    )
    token, expires_at = create_access_token(
        settings,
        subject=user.external_subject,
        identity_provider=user.identity_provider,
        display_name=user.display_name,
        email=user.email,
    )
    set_session_cookie(response, settings, token=token, expires_at=expires_at)
    return AccessTokenResponse(access_token=token, expires_at=expires_at)


@router.post(
    "/auth/logout",
    status_code=status.HTTP_204_NO_CONTENT,
    summary="Clear the session cookie",
    description="Expires the browser session cookie. Always reports success.",
)
async def logout(settings: Settings = Depends(get_settings)) -> Response:
    response = Response(status_code=status.HTTP_204_NO_CONTENT)
    clear_session_cookie(response, settings)
    return response
