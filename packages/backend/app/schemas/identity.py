"""Request and response models for identity, organisations, and memberships."""

from __future__ import annotations

from typing import Literal
from uuid import UUID

from pydantic import BaseModel, Field, field_validator

from app.models.identity import Role
from app.schemas.common import RequestModel, UtcTimestamp


class DevelopmentTokenRequest(RequestModel):
    """Optional overrides for the development token subject."""

    subject: str | None = Field(default=None, max_length=200)
    identity_provider: str | None = Field(default=None, max_length=100)
    display_name: str | None = Field(default=None, max_length=200)
    email: str | None = Field(default=None, max_length=320)

    @field_validator("subject", "identity_provider", mode="before")
    @classmethod
    def _strip_requiredish(cls, value: object) -> object:
        return value.strip() if isinstance(value, str) else value


class AccessTokenResponse(BaseModel):
    access_token: str
    token_type: Literal["Bearer"] = "Bearer"
    expires_at: UtcTimestamp


class MembershipCreateRequest(RequestModel):
    identity_provider: str = Field(min_length=1, max_length=100)
    subject: str = Field(min_length=1, max_length=200)
    role: Role
    display_name: str | None = Field(default=None, max_length=200)
    email: str | None = Field(default=None, max_length=320)

    @field_validator("identity_provider", "subject", mode="before")
    @classmethod
    def _strip_identity(cls, value: object) -> object:
        return value.strip() if isinstance(value, str) else value


class MembershipRoleUpdateRequest(RequestModel):
    role: Role


class MembershipSummary(BaseModel):
    membership_id: UUID
    organisation_id: UUID
    organisation_name: str
    role: Role


class CurrentUserResponse(BaseModel):
    id: UUID
    display_name: str | None
    email: str | None
    memberships: list[MembershipSummary]


class OrganisationSummary(BaseModel):
    id: UUID
    name: str
    slug: str | None
    status: str
    role: Role


class OrganisationListResponse(BaseModel):
    items: list[OrganisationSummary]
    next_cursor: str | None = None
    has_more: bool = False


class OrganisationDetailResponse(BaseModel):
    id: UUID
    name: str
    slug: str | None
    status: str
    role: Role
    created_at: UtcTimestamp
    updated_at: UtcTimestamp


class OrganisationCreateRequest(RequestModel):
    """Request to create a new organisation."""

    name: str = Field(min_length=1, max_length=200)
    slug: str | None = Field(default=None, max_length=200)

    @field_validator("name", "slug", mode="before")
    @classmethod
    def _strip_organisation(cls, value: object) -> object:
        return value.strip() if isinstance(value, str) else value


class MembershipRead(BaseModel):
    id: UUID
    user_id: UUID
    display_name: str | None
    email: str | None
    role: Role
    created_at: UtcTimestamp
    updated_at: UtcTimestamp


class MembershipListResponse(BaseModel):
    items: list[MembershipRead]
    next_cursor: str | None = None
    has_more: bool = False
