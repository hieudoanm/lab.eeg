"""Unit tests for access-token creation and verification."""

from __future__ import annotations

from datetime import UTC, datetime, timedelta

import jwt
import pytest
from app.core.exceptions import UnauthorizedError
from app.core.security import create_access_token, decode_access_token

from tests.conftest import make_settings


def test_round_trip_preserves_claims() -> None:
    settings = make_settings()
    token, expires_at = create_access_token(
        settings,
        subject="researcher-1",
        identity_provider="local",
        display_name="Ada",
        email="ada@example.test",
    )
    claims = decode_access_token(settings, token)
    assert claims["sub"] == "researcher-1"
    assert claims["idp"] == "local"
    assert claims["name"] == "Ada"
    assert claims["iss"] == settings.auth_token_issuer
    assert expires_at > datetime.now(UTC)


def test_expired_token_is_rejected() -> None:
    settings = make_settings().model_copy(update={"auth_token_ttl_seconds": -10})
    token, _ = create_access_token(settings, subject="researcher-1", identity_provider="local")
    with pytest.raises(UnauthorizedError) as excinfo:
        decode_access_token(settings, token)
    assert excinfo.value.code == "token_expired"
    assert excinfo.value.headers["WWW-Authenticate"] == "Bearer"


def test_tampered_signature_is_rejected() -> None:
    settings = make_settings()
    token, _ = create_access_token(settings, subject="researcher-1", identity_provider="local")
    with pytest.raises(UnauthorizedError) as excinfo:
        decode_access_token(settings, f"{token}tampered")
    assert excinfo.value.code == "invalid_token"


def test_wrong_audience_is_rejected() -> None:
    issuer = make_settings()
    token, _ = create_access_token(issuer, subject="researcher-1", identity_provider="local")
    verifier = issuer.model_copy(update={"auth_token_audience": "another-api"})
    with pytest.raises(UnauthorizedError):
        decode_access_token(verifier, token)


def test_wrong_issuer_is_rejected() -> None:
    issuer = make_settings()
    token, _ = create_access_token(issuer, subject="researcher-1", identity_provider="local")
    verifier = issuer.model_copy(update={"auth_token_issuer": "somebody-else"})
    with pytest.raises(UnauthorizedError):
        decode_access_token(verifier, token)


def test_token_without_identity_provider_claim_is_rejected() -> None:
    settings = make_settings()
    now = datetime.now(UTC)
    token = jwt.encode(
        {
            "sub": "researcher-1",
            "iss": settings.auth_token_issuer,
            "aud": settings.auth_token_audience,
            "iat": int(now.timestamp()),
            "exp": int((now + timedelta(hours=1)).timestamp()),
        },
        settings.auth_token_secret,
        algorithm="HS256",
    )
    with pytest.raises(UnauthorizedError) as excinfo:
        decode_access_token(settings, token)
    assert excinfo.value.code == "invalid_token"
