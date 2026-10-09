"""HTTP error handlers producing the documented error envelope."""

from __future__ import annotations

import logging
from typing import Any

from fastapi import FastAPI, Request
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse
from starlette.exceptions import HTTPException as StarletteHTTPException

from app.core.exceptions import AppError, ErrorDetail
from app.core.logging import REQUEST_ID_CONTEXT
from app.schemas.errors import ErrorBody, ErrorDetailModel, ErrorResponse

logger = logging.getLogger("eeg-lab.api.errors")

_STATUS_CODE_MAP: dict[int, str] = {
    400: "bad_request",
    401: "unauthorized",
    403: "forbidden",
    404: "not_found",
    405: "method_not_allowed",
    409: "conflict",
    413: "payload_too_large",
    415: "unsupported_media_type",
    422: "validation_error",
    429: "rate_limit_exceeded",
    500: "internal_error",
    503: "service_unavailable",
}


def _request_id(request: Request) -> str | None:
    return REQUEST_ID_CONTEXT.get() or getattr(request.state, "request_id", None)


def error_response(
    request: Request,
    *,
    status_code: int,
    code: str,
    message: str,
    details: list[ErrorDetailModel] | None = None,
    extra_headers: dict[str, str] | None = None,
) -> JSONResponse:
    """Build a JSON error envelope response."""
    body = ErrorResponse(
        error=ErrorBody(
            code=code,
            message=message,
            details=details or [],
            request_id=_request_id(request),
        )
    )
    headers: dict[str, str] = dict(extra_headers or {})
    request_id = _request_id(request)
    if request_id:
        headers["X-Request-Id"] = request_id
    return JSONResponse(
        status_code=status_code, content=body.model_dump(exclude_none=True), headers=headers
    )


def install_error_handlers(app: FastAPI) -> None:
    """Register exception handlers on the application."""

    @app.exception_handler(AppError)
    async def _app_error(request: Request, exc: AppError) -> JSONResponse:
        details = [ErrorDetailModel(**detail.to_dict()) for detail in exc.details]
        if exc.status_code >= 500:
            logger.exception("application error", extra={"error_code": exc.code})
        else:
            logger.info(
                "request rejected",
                extra={"error_code": exc.code, "status_code": exc.status_code},
            )
        return error_response(
            request,
            status_code=exc.status_code,
            code=exc.code,
            message=exc.message,
            details=details,
            extra_headers=exc.headers,
        )

    @app.exception_handler(RequestValidationError)
    async def _validation_error(request: Request, exc: RequestValidationError) -> JSONResponse:
        details = [
            ErrorDetailModel(
                field=_format_field(error),
                code=error["type"],
                message=_format_message(error),
            )
            for error in exc.errors()
        ]
        return error_response(
            request,
            status_code=422,
            code="validation_error",
            message="The request contains invalid fields.",
            details=details,
        )

    @app.exception_handler(StarletteHTTPException)
    async def _http_error(request: Request, exc: StarletteHTTPException) -> JSONResponse:
        code = _STATUS_CODE_MAP.get(exc.status_code or 500, "http_error")
        message = (
            exc.detail if isinstance(exc.detail, str) else "The request could not be completed."
        )
        return error_response(
            request,
            status_code=exc.status_code or 500,
            code=code,
            message=message,
        )

    @app.exception_handler(Exception)
    async def _unhandled_error(request: Request, exc: Exception) -> JSONResponse:
        logger.exception("unhandled error", extra={"path": request.url.path})
        return error_response(
            request,
            status_code=500,
            code="internal_error",
            message=(
                "An unexpected error occurred. "
                "Provide the request identifier when reporting this problem."
            ),
        )


def _format_field(error: dict[str, Any]) -> str:
    location = error.get("loc") or ()
    parts = [str(part) for part in location if part != "body"]
    return ".".join(parts) if parts else "body"


def _format_message(error: dict[str, Any]) -> str:
    message = error.get("msg", "Invalid value.")
    return str(message).replace("Value error, ", "")


__all__ = ["ErrorDetail", "error_response", "install_error_handlers"]
