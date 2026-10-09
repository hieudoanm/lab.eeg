"""Unit tests for configuration loading and validation."""

from __future__ import annotations

import pytest

from tests.conftest import make_settings


def test_defaults_are_test_safe() -> None:
    settings = make_settings()
    assert settings.app_env == "test"
    assert settings.auth_mode == "development"
    assert settings.default_page_size == 20
    assert settings.max_page_size == 100
    assert settings.max_signal_points_per_request == 20_000


def test_cors_origins_are_split_from_a_string() -> None:
    settings = make_settings(api_cors_origins="http://a.test, http://b.test")
    assert settings.api_cors_origins == ["http://a.test", "http://b.test"]


def test_log_level_is_normalised() -> None:
    assert make_settings(log_level="debug").log_level == "DEBUG"


def test_non_postgres_database_url_is_rejected() -> None:
    with pytest.raises(ValueError):
        make_settings(database_url="sqlite:///eeg-lab.db")


def test_production_rejects_development_authentication() -> None:
    with pytest.raises(ValueError, match="AUTH_MODE"):
        make_settings(app_env="production", auth_mode="development", log_format="json")


def test_production_rejects_console_logging() -> None:
    with pytest.raises(ValueError, match="LOG_FORMAT"):
        make_settings(app_env="production", auth_mode="oidc", log_format="console")


def test_production_accepts_hardened_configuration() -> None:
    settings = make_settings(
        app_env="production",
        auth_mode="oidc",
        log_format="json",
        auth_token_secret="k" * 40,
        database_pool_min_size=2,
        database_pool_max_size=4,
    )
    assert settings.is_production is True
    assert settings.development_auth_enabled is False


def test_development_auth_endpoint_is_enabled_in_development() -> None:
    assert make_settings().development_auth_enabled is True


def test_production_rejects_default_auth_secret() -> None:
    with pytest.raises(ValueError, match="AUTH_TOKEN_SECRET"):
        make_settings(app_env="production", auth_mode="oidc", log_format="json")


def test_production_rejects_short_auth_secret() -> None:
    with pytest.raises(ValueError, match="AUTH_TOKEN_SECRET"):
        make_settings(
            app_env="production",
            auth_mode="oidc",
            log_format="json",
            auth_token_secret="too-short",
        )


def test_asymmetric_algorithms_require_a_public_key() -> None:
    with pytest.raises(ValueError, match="AUTH_PUBLIC_KEY"):
        make_settings(
            app_env="production",
            auth_mode="oidc",
            log_format="json",
            auth_jwt_algorithms="RS256",
        )


def test_token_ttl_must_be_positive() -> None:
    with pytest.raises(ValueError, match="AUTH_TOKEN_TTL_SECONDS"):
        make_settings(auth_token_ttl_seconds=0)


def test_jwt_algorithms_are_split_from_a_string() -> None:
    settings = make_settings(auth_jwt_algorithms="HS256, RS256")
    assert settings.auth_jwt_algorithms == ["HS256", "RS256"]


def test_samesite_none_requires_a_secure_cookie() -> None:
    with pytest.raises(ValueError, match="AUTH_COOKIE_SAMESITE"):
        make_settings(auth_cookie_samesite="none")


def test_production_uses_secure_cookies_by_default() -> None:
    settings = make_settings(
        app_env="production",
        auth_mode="oidc",
        log_format="json",
        auth_token_secret="k" * 40,
    )
    assert settings.resolved_auth_cookie_secure is True


def test_blank_cookie_name_is_rejected() -> None:
    with pytest.raises(ValueError, match="AUTH_COOKIE_NAME"):
        make_settings(auth_cookie_name="  ")


def test_blank_development_organisation_name_is_rejected() -> None:
    with pytest.raises(ValueError, match="DEV_ORGANISATION_NAME"):
        make_settings(dev_organisation_name="  ")


def test_invalid_pool_bounds_are_rejected_in_production() -> None:
    with pytest.raises(ValueError, match="DATABASE_POOL_MAX_SIZE"):
        make_settings(
            app_env="production",
            auth_mode="oidc",
            log_format="json",
            database_pool_min_size=10,
            database_pool_max_size=5,
        )


def test_cached_settings_can_be_reset() -> None:
    from app.core.config import clear_settings_cache, get_settings

    first = get_settings()
    clear_settings_cache()
    second = get_settings()
    assert first is not second
