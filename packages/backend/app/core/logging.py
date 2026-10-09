"""Structured logging with request and job correlation identifiers."""

from __future__ import annotations

import json
import logging
import sys
from contextvars import ContextVar, Token
from typing import Any

REQUEST_ID_CONTEXT: ContextVar[str | None] = ContextVar("request_id", default=None)
JOB_ID_CONTEXT: ContextVar[str | None] = ContextVar("job_id", default=None)

_RESERVED_RECORD_KEYS = frozenset(
    {
        "args",
        "asctime",
        "created",
        "exc_info",
        "exc_text",
        "filename",
        "funcName",
        "levelname",
        "levelno",
        "lineno",
        "module",
        "msecs",
        "message",
        "msg",
        "name",
        "pathname",
        "process",
        "processName",
        "relativeCreated",
        "stack_info",
        "thread",
        "threadName",
        "taskName",
    }
)


class JsonFormatter(logging.Formatter):
    """Format log records as single-line JSON objects."""

    def format(self, record: logging.LogRecord) -> str:
        payload: dict[str, Any] = {
            "timestamp": self.formatTime(record, "%Y-%m-%dT%H:%M:%S.%03dZ"),
            "severity": record.levelname,
            "logger": record.name,
            "service": getattr(record, "service", "eeg-lab-api"),
            "message": record.getMessage(),
        }

        request_id = REQUEST_ID_CONTEXT.get()
        if request_id:
            payload["request_id"] = request_id
        job_id = JOB_ID_CONTEXT.get()
        if job_id:
            payload["job_id"] = job_id

        for key, value in record.__dict__.items():
            if key in _RESERVED_RECORD_KEYS or key.startswith("_"):
                continue
            payload.setdefault(key, value)

        if record.exc_info:
            payload["exception"] = self.formatException(record.exc_info)

        return json.dumps(payload, default=str, ensure_ascii=True)


class ConsoleFormatter(logging.Formatter):
    """Human-readable output for local development."""

    def format(self, record: logging.LogRecord) -> str:
        request_id = REQUEST_ID_CONTEXT.get()
        correlation = f" [{request_id}]" if request_id else ""
        base = (
            f"{self.formatTime(record, '%H:%M:%S')} "
            f"{record.levelname:<7} {record.name}{correlation} {record.getMessage()}"
        )
        if record.exc_info:
            base = f"{base}\n{self.formatException(record.exc_info)}"
        return base


def configure_logging(level: str = "INFO", fmt: str = "json") -> None:
    """Install a single structured handler on the root logger."""
    handler = logging.StreamHandler(sys.stdout)
    handler.setFormatter(JsonFormatter() if fmt == "json" else ConsoleFormatter())

    root = logging.getLogger()
    root.handlers.clear()
    root.addHandler(handler)
    root.setLevel(level.upper())

    # uvicorn attaches its own handlers; keep our formatter authoritative.
    for logger_name in ("uvicorn", "uvicorn.error", "uvicorn.access"):
        named = logging.getLogger(logger_name)
        named.handlers.clear()
        named.propagate = True


def bind_request_id(request_id: str) -> Token[str | None]:
    """Bind a request identifier to the current context."""
    return REQUEST_ID_CONTEXT.set(request_id)


def reset_request_id(token: Token[str | None]) -> None:
    """Restore the previous request identifier context."""
    REQUEST_ID_CONTEXT.reset(token)


def bind_job_id(job_id: str) -> Token[str | None]:
    """Bind a job identifier to the current context."""
    return JOB_ID_CONTEXT.set(job_id)


def reset_job_id(token: Token[str | None]) -> None:
    """Restore the previous job identifier context."""
    JOB_ID_CONTEXT.reset(token)
