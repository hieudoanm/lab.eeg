"""HTTP middleware: request identifiers and request body limits."""

from __future__ import annotations

import logging
import re
import time
import uuid
from collections.abc import Awaitable, Callable

from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request
from starlette.responses import Response
from starlette.types import ASGIApp

from app.api.errors import error_response
from app.core.config import Settings
from app.core.logging import bind_request_id, reset_request_id

logger = logging.getLogger("eeg-lab.api.requests")

_SAFE_REQUEST_ID = re.compile(r"^[A-Za-z0-9._-]{1,64}$")

NextHandler = Callable[[Request], Awaitable[Response]]


def resolve_request_id(header_value: str | None) -> str:
    """Reuse a well-formed client correlation id, otherwise generate one."""
    if header_value and _SAFE_REQUEST_ID.match(header_value):
        return header_value
    return uuid.uuid4().hex


class RequestContextMiddleware(BaseHTTPMiddleware):
    """Attach a correlation identifier and enforce the request body limit."""

    def __init__(self, app: ASGIApp, settings: Settings) -> None:
        super().__init__(app)
        self._settings = settings

    async def dispatch(self, request: Request, call_next: NextHandler) -> Response:
        request_id = resolve_request_id(request.headers.get("X-Request-Id"))
        request.state.request_id = request_id
        token = bind_request_id(request_id)

        content_length = request.headers.get("Content-Length")
        path = request.url.path
        limit = self._settings.max_request_body_bytes
        # Allow larger body for upload content endpoint
        is_upload_content = (
            path.startswith("/api/v1/organisations/")
            and path.endswith("/content")
            and request.method == "PUT"
        )
        if is_upload_content:
            limit = self._settings.max_upload_bytes

        if content_length is not None and content_length.isdigit() and int(content_length) > limit:
            reset_request_id(token)
            return error_response(
                request,
                status_code=413,
                code="payload_too_large",
                message=(f"The request body exceeds the configured limit of {limit} bytes."),
            )

        start = time.perf_counter()
        try:
            response = await call_next(request)
        except Exception:
            logger.exception(
                "request failed",
                extra={
                    "method": request.method,
                    "path": request.url.path,
                    "duration_ms": round((time.perf_counter() - start) * 1000, 2),
                },
            )
            reset_request_id(token)
            raise

        duration_ms = round((time.perf_counter() - start) * 1000, 2)
        route = request.scope.get("route")
        logger.info(
            "request completed",
            extra={
                "method": request.method,
                "path": request.url.path,
                "route": getattr(route, "path", None),
                "status_code": response.status_code,
                "duration_ms": duration_ms,
            },
        )
        response.headers["X-Request-Id"] = request_id
        reset_request_id(token)
        return response
