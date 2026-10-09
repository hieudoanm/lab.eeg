"""Public error response models."""

from __future__ import annotations

from pydantic import BaseModel, Field


class ErrorDetailModel(BaseModel):
    """Field-level validation problem."""

    field: str | None = None
    code: str
    message: str


class ErrorBody(BaseModel):
    code: str = Field(description="Stable machine-readable error code")
    message: str = Field(description="Human-readable message safe for display")
    details: list[ErrorDetailModel] = Field(default_factory=list)
    request_id: str | None = None


class ErrorResponse(BaseModel):
    """Error envelope returned by every non-2xx response."""

    error: ErrorBody
