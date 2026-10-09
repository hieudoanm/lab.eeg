"""Unit tests for request identifier handling."""

from __future__ import annotations

from app.api.middleware import resolve_request_id


def test_safe_header_is_reused() -> None:
    assert resolve_request_id("caller-abc_123.4") == "caller-abc_123.4"


def test_missing_header_generates_identifier() -> None:
    request_id = resolve_request_id(None)
    assert len(request_id) == 32
    assert request_id.isalnum()


def test_log_injection_attempt_is_replaced() -> None:
    malicious = 'abc\n{"severity": "CRITICAL"}'
    generated = resolve_request_id(malicious)
    assert generated != malicious
    assert "\n" not in generated


def test_overlong_header_is_replaced() -> None:
    assert resolve_request_id("x" * 65) != "x" * 65
