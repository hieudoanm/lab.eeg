"""Session cookie helpers for browser clients.

The access token is also returned in the response body so API and test clients
can use it directly, but browsers are expected to rely on the HttpOnly cookie.
"""

from __future__ import annotations

from datetime import UTC, datetime

from fastapi import Response

from app.core.config import Settings


def set_session_cookie(
    response: Response,
    settings: Settings,
    *,
    token: str,
    expires_at: datetime,
) -> None:
    """Attach the session cookie to a response."""
    max_age = max(0, int((expires_at - datetime.now(UTC)).total_seconds()))
    response.set_cookie(
        key=settings.auth_cookie_name,
        value=token,
        max_age=max_age,
        httponly=True,
        secure=settings.resolved_auth_cookie_secure,
        samesite=settings.auth_cookie_samesite,
        domain=settings.auth_cookie_domain,
        path="/",
    )


def clear_session_cookie(response: Response, settings: Settings) -> None:
    """Expire the session cookie on the client."""
    response.delete_cookie(
        key=settings.auth_cookie_name,
        domain=settings.auth_cookie_domain,
        path="/",
    )
