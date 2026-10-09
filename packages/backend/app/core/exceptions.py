"""Application exceptions carrying stable, safe error codes."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import Any


class ErrorDetail:
    """A single machine-readable field level problem."""

    __slots__ = ("code", "field", "message")

    def __init__(self, message: str, *, code: str = "invalid", field: str | None = None) -> None:
        self.message = message
        self.code = code
        self.field = field

    def to_dict(self) -> dict[str, Any]:
        payload: dict[str, Any] = {"code": self.code, "message": self.message}
        if self.field is not None:
            payload["field"] = self.field
        return payload


class AppError(Exception):
    """Base class for errors that map onto the documented API error envelope."""

    status_code: int = 500
    code: str = "internal_error"
    default_message: str = "The request could not be completed."

    def __init__(
        self,
        message: str | None = None,
        *,
        code: str | None = None,
        details: Sequence[ErrorDetail] | None = None,
        headers: Mapping[str, str] | None = None,
    ) -> None:
        self.message = message or self.default_message
        if code is not None:
            self.code = code
        self.details: list[ErrorDetail] = list(details or [])
        self.headers: dict[str, str] = dict(headers or {})
        super().__init__(self.message)


class BadRequestError(AppError):
    status_code = 400
    code = "bad_request"
    default_message = "The request is malformed."


class UnauthorizedError(AppError):
    status_code = 401
    code = "unauthorized"
    default_message = "Authentication is required."

    def __init__(
        self,
        message: str | None = None,
        *,
        code: str | None = None,
        details: Sequence[ErrorDetail] | None = None,
        headers: Mapping[str, str] | None = None,
    ) -> None:
        merged = {"WWW-Authenticate": "Bearer"}
        if headers is not None:
            merged.update(headers)
        super().__init__(message, code=code, details=details, headers=merged)


class ForbiddenError(AppError):
    status_code = 403
    code = "forbidden"
    default_message = "You do not have permission to perform this action."


class NotFoundError(AppError):
    status_code = 404
    code = "not_found"
    default_message = "The requested resource does not exist."


class ConflictError(AppError):
    status_code = 409
    code = "conflict"
    default_message = "The request conflicts with the current resource state."


class PayloadTooLargeError(AppError):
    status_code = 413
    code = "payload_too_large"
    default_message = "The request body exceeds the configured limit."


class UnsupportedMediaTypeError(AppError):
    status_code = 415
    code = "unsupported_media_type"
    default_message = "The requested media type is not supported."


class UnprocessableEntityError(AppError):
    status_code = 422
    code = "validation_error"
    default_message = "The request contains invalid fields."


class RateLimitExceededError(AppError):
    status_code = 429
    code = "rate_limit_exceeded"
    default_message = "Too many requests. Please retry later."


class ServiceUnavailableError(AppError):
    status_code = 503
    code = "service_unavailable"
    default_message = "A required dependency is temporarily unavailable."
