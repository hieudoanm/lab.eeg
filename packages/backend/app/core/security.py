"""Access-token creation and verification.

Tokens are signed JWTs. The development token endpoint mints short-lived tokens
with the configured HMAC secret; production deployments validate tokens issued
by the configured identity provider (HMAC or asymmetric algorithms).
"""

from __future__ import annotations

import uuid
from datetime import UTC, datetime, timedelta
from typing import Any

import jwt

from app.core.config import Settings
from app.core.exceptions import UnauthorizedError

_ASYMMETRIC_PREFIXES = ("RS", "ES", "PS")
_REQUIRED_CLAIMS = ["exp", "iat", "sub", "iss", "aud"]


def _signing_key(settings: Settings) -> str:
    """Select the verification/signing key for the configured algorithms."""
    if any(
        algorithm.startswith(_ASYMMETRIC_PREFIXES) for algorithm in settings.auth_jwt_algorithms
    ):
        if not settings.auth_public_key:
            raise UnauthorizedError(
                "Authentication is not correctly configured.",
                code="auth_misconfigured",
            )
        return settings.auth_public_key
    return settings.auth_token_secret


def create_access_token(
    settings: Settings,
    *,
    subject: str,
    identity_provider: str,
    display_name: str | None = None,
    email: str | None = None,
) -> tuple[str, datetime]:
    """Mint a signed access token, returning it with its expiry instant."""
    issued_at = datetime.now(UTC)
    expires_at = issued_at + timedelta(seconds=settings.auth_token_ttl_seconds)
    claims: dict[str, Any] = {
        "sub": subject,
        "idp": identity_provider,
        "iss": settings.auth_token_issuer,
        "aud": settings.auth_token_audience,
        "iat": int(issued_at.timestamp()),
        "exp": int(expires_at.timestamp()),
        "jti": uuid.uuid4().hex,
    }
    if display_name:
        claims["name"] = display_name
    if email:
        claims["email"] = email
    algorithm = settings.auth_jwt_algorithms[0]
    token = jwt.encode(claims, _signing_key(settings), algorithm=algorithm)
    return token, expires_at


def decode_access_token(settings: Settings, token: str) -> dict[str, Any]:
    """Verify a token and return its claims, raising ``UnauthorizedError`` otherwise."""
    try:
        claims: dict[str, Any] = jwt.decode(
            token,
            _signing_key(settings),
            algorithms=list(settings.auth_jwt_algorithms),
            audience=settings.auth_token_audience,
            issuer=settings.auth_token_issuer,
            options={"require": _REQUIRED_CLAIMS},
        )
    except jwt.ExpiredSignatureError as exc:
        raise UnauthorizedError("The access token has expired.", code="token_expired") from exc
    except jwt.InvalidTokenError as exc:
        raise UnauthorizedError("The access token is invalid.", code="invalid_token") from exc

    subject = claims.get("sub")
    provider = claims.get("idp")
    if not isinstance(subject, str) or not subject or not isinstance(provider, str) or not provider:
        raise UnauthorizedError("The access token is invalid.", code="invalid_token")
    return claims
