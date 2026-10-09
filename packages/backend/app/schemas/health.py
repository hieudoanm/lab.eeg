"""Health and readiness response models."""

from __future__ import annotations

from typing import Literal

from pydantic import BaseModel, Field


class HealthLiveResponse(BaseModel):
    status: Literal["alive"] = "alive"
    service: str
    version: str
    environment: str


class HealthReadyResponse(BaseModel):
    status: Literal["ready", "unavailable"]
    service: str
    version: str
    checks: dict[str, str] = Field(default_factory=dict)
