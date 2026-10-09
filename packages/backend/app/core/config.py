"""Application configuration.

All configuration is environment driven. Required values are validated at import
time so that a misconfigured process fails fast instead of failing per request.
"""

from __future__ import annotations

from functools import lru_cache
from typing import Literal

from pydantic import Field, field_validator
from pydantic_settings import BaseSettings, SettingsConfigDict

Environment = Literal["development", "test", "production"]
AuthMode = Literal["development", "oidc"]
AuthCookieSameSite = Literal["lax", "strict", "none"]
StorageBackend = Literal["local", "gcs"]
LogFormat = Literal["json", "console"]

#: Sentinel default for the development token secret. Production must override it.
DEFAULT_AUTH_TOKEN_SECRET = "insecure-development-secret-change-me"
_MIN_AUTH_SECRET_LENGTH = 32


class Settings(BaseSettings):
    """Runtime configuration for the API process."""

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        extra="ignore",
        case_sensitive=False,
    )

    # Identity of the process -------------------------------------------------
    app_env: Environment = "development"
    app_name: str = "eeg-lab-api"
    app_version: str = "0.1.0"
    log_level: str = "DEBUG"
    log_format: LogFormat = "json"

    # HTTP -------------------------------------------------------------------
    api_cors_origins: list[str] = Field(default_factory=lambda: ["http://localhost:3000"])
    max_request_body_bytes: int = 1_048_576

    # Database ---------------------------------------------------------------
    database_url: str = "postgresql://eeg-lab:eeg-lab@localhost:5432/eeg-lab"
    database_pool_min_size: int = 1
    database_pool_max_size: int = 10
    database_connect_timeout_seconds: float = 5.0
    database_statement_timeout_ms: int = 15_000

    # Pagination -------------------------------------------------------------
    default_page_size: int = 20
    max_page_size: int = 100

    # Storage ----------------------------------------------------------------
    storage_backend: StorageBackend = "local"
    local_storage_path: str = "./.local-data/uploads"
    gcs_bucket: str | None = None
    gcs_project: str | None = None

    # Authentication ---------------------------------------------------------
    auth_mode: AuthMode = "development"
    auth_token_secret: str = DEFAULT_AUTH_TOKEN_SECRET
    auth_public_key: str | None = None
    auth_jwt_algorithms: list[str] = Field(default_factory=lambda: ["HS256"])
    auth_token_issuer: str = "eeg-lab"
    auth_token_audience: str = "eeg-lab-api"
    auth_token_ttl_seconds: int = 3600

    # Session cookie (browser clients) ---------------------------------------
    auth_cookie_name: str = "eeg-lab_session"
    auth_cookie_secure: bool | None = None
    auth_cookie_samesite: AuthCookieSameSite = "lax"
    auth_cookie_domain: str | None = None

    # Development identity defaults (used only when AUTH_MODE=development) ----
    dev_identity_provider: str = "local"
    dev_identity_subject: str = "demo-researcher"
    dev_identity_display_name: str = "Demo Researcher"
    dev_identity_email: str | None = "demo@eeg-lab.local"
    dev_organisation_name: str = "Development Organisation"

    # Resource limits --------------------------------------------------------
    max_upload_bytes: int = 268_435_456
    max_channels: int = 256
    max_sample_count: int = 100_000_000
    max_signal_window_seconds: float = 300.0
    max_signal_points_per_request: int = 20_000

    # Upload sessions ---------------------------------------------------------
    upload_session_ttl_seconds: int = 3600

    # Background processing --------------------------------------------------
    redis_url: str = "redis://localhost:6379/0"
    celery_broker_url: str | None = None
    celery_task_time_limit_seconds: int = 300
    celery_task_soft_time_limit_seconds: int = 280
    job_lease_seconds: int = 120
    analysis_max_attempts: int = 3
    outbox_dispatch_max_attempts: int = 10

    # Idempotency ------------------------------------------------------------
    idempotency_ttl_seconds: int = 86_400

    @field_validator("api_cors_origins", mode="before")
    @classmethod
    def _split_origins(cls, value: object) -> object:
        if isinstance(value, str):
            return [origin.strip() for origin in value.split(",") if origin.strip()]
        return value

    @field_validator("auth_jwt_algorithms", mode="before")
    @classmethod
    def _split_algorithms(cls, value: object) -> object:
        if isinstance(value, str):
            return [item.strip() for item in value.split(",") if item.strip()]
        return value

    @field_validator("log_level")
    @classmethod
    def _normalise_log_level(cls, value: str) -> str:
        return value.upper()

    @field_validator("dev_organisation_name", mode="before")
    @classmethod
    def _strip_dev_organisation_name(cls, value: object) -> object:
        return value.strip() if isinstance(value, str) else value

    @field_validator("database_url")
    @classmethod
    def _require_postgres(cls, value: str) -> str:
        if not value.startswith(("postgresql://", "postgres://")):
            raise ValueError("DATABASE_URL must be a PostgreSQL connection string")
        return value

    def model_post_init(self, __context: object) -> None:
        """Fail closed when development-only behaviour is enabled in production."""
        if self.auth_token_ttl_seconds <= 0:
            raise ValueError("AUTH_TOKEN_TTL_SECONDS must be greater than zero")
        if self.upload_session_ttl_seconds <= 0:
            raise ValueError("UPLOAD_SESSION_TTL_SECONDS must be greater than zero")
        if self.idempotency_ttl_seconds <= 0:
            raise ValueError("IDEMPOTENCY_TTL_SECONDS must be greater than zero")
        if self.analysis_max_attempts <= 0:
            raise ValueError("ANALYSIS_MAX_ATTEMPTS must be greater than zero")
        if self.outbox_dispatch_max_attempts <= 0:
            raise ValueError("OUTBOX_DISPATCH_MAX_ATTEMPTS must be greater than zero")
        if self.celery_task_time_limit_seconds <= 0:
            raise ValueError("CELERY_TASK_TIME_LIMIT_SECONDS must be greater than zero")
        if not (
            0 < self.celery_task_soft_time_limit_seconds <= self.celery_task_time_limit_seconds
        ):
            raise ValueError(
                "CELERY_TASK_SOFT_TIME_LIMIT_SECONDS must be positive and no greater than "
                "CELERY_TASK_TIME_LIMIT_SECONDS"
            )
        if not self.auth_cookie_name.strip():
            raise ValueError("AUTH_COOKIE_NAME must not be blank")
        if self.auth_cookie_samesite == "none" and not self.resolved_auth_cookie_secure:
            raise ValueError("AUTH_COOKIE_SAMESITE=none requires a secure cookie")
        if self.development_auth_enabled and not self.dev_organisation_name:
            raise ValueError(
                "DEV_ORGANISATION_NAME must not be blank when development auth is enabled"
            )
        if self.app_env == "production":
            if self.auth_mode == "development":
                raise ValueError("AUTH_MODE=development is not permitted when APP_ENV=production")
            if self.log_format == "console":
                raise ValueError("LOG_FORMAT must be 'json' when APP_ENV=production")
            if self.database_pool_max_size < self.database_pool_min_size:
                raise ValueError("DATABASE_POOL_MAX_SIZE must be >= DATABASE_POOL_MIN_SIZE")
            self._validate_production_auth_key()

    def _validate_production_auth_key(self) -> None:
        """Reject insecure default signing configuration in production."""
        uses_hmac = any(algorithm.startswith("HS") for algorithm in self.auth_jwt_algorithms)
        uses_public_key = any(
            algorithm.startswith(("RS", "ES", "PS")) for algorithm in self.auth_jwt_algorithms
        )
        if uses_hmac:
            if self.auth_token_secret == DEFAULT_AUTH_TOKEN_SECRET:
                raise ValueError(
                    "AUTH_TOKEN_SECRET must be set to a non-default value when APP_ENV=production"
                )
            if len(self.auth_token_secret) < _MIN_AUTH_SECRET_LENGTH:
                raise ValueError(
                    f"AUTH_TOKEN_SECRET must be at least {_MIN_AUTH_SECRET_LENGTH} characters"
                )
        if uses_public_key and not self.auth_public_key:
            raise ValueError("AUTH_PUBLIC_KEY must be configured for asymmetric JWT algorithms")

    @property
    def is_production(self) -> bool:
        return self.app_env == "production"

    @property
    def is_development(self) -> bool:
        return self.app_env == "development"

    @property
    def development_auth_enabled(self) -> bool:
        """True only when the development token endpoint may be served."""
        return self.auth_mode == "development" and self.app_env != "production"

    @property
    def resolved_auth_cookie_secure(self) -> bool:
        """Use secure cookies by default in production, allowing an explicit override."""
        if self.auth_cookie_secure is not None:
            return self.auth_cookie_secure
        return self.app_env == "production"

    @property
    def resolved_celery_broker_url(self) -> str:
        """The Celery broker URL, defaulting to the configured Redis instance."""
        return self.celery_broker_url or self.redis_url


@lru_cache(maxsize=1)
def get_settings() -> Settings:
    """Return the process-wide settings instance."""
    return Settings()


def clear_settings_cache() -> None:
    """Reset the cached settings (used by tests)."""
    get_settings.cache_clear()
