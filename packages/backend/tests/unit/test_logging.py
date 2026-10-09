"""Unit tests for structured logging helpers."""

from __future__ import annotations

import json
import logging

from app.core.logging import (
    JsonFormatter,
    bind_job_id,
    bind_request_id,
    reset_job_id,
    reset_request_id,
)


def _record(message: str = "hello") -> logging.LogRecord:
    return logging.LogRecord("eeg-lab.test", logging.INFO, __file__, 1, message, None, None)


def test_json_formatter_produces_structured_output() -> None:
    payload = json.loads(JsonFormatter().format(_record()))
    assert payload["severity"] == "INFO"
    assert payload["logger"] == "eeg-lab.test"
    assert payload["message"] == "hello"
    assert payload["service"] == "eeg-lab-api"


def test_json_formatter_includes_request_and_job_identifiers() -> None:
    request_token = bind_request_id("req_abc")
    job_token = bind_job_id("job_123")
    try:
        payload = json.loads(JsonFormatter().format(_record()))
    finally:
        reset_request_id(request_token)
        reset_job_id(job_token)
    assert payload["request_id"] == "req_abc"
    assert payload["job_id"] == "job_123"


def test_json_formatter_serialises_extra_fields() -> None:
    record = _record()
    record.status_code = 503
    payload = json.loads(JsonFormatter().format(record))
    assert payload["status_code"] == 503


def test_json_formatter_omits_identifiers_when_unbound() -> None:
    payload = json.loads(JsonFormatter().format(_record()))
    assert "request_id" not in payload
    assert "job_id" not in payload
