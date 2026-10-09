"""Identity, organisation, and membership domain models.

These dataclasses are the boundary between explicit SQL repositories and the
application/API layers. API responses are built from dedicated Pydantic schemas
rather than exposing these objects directly.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from enum import StrEnum
from uuid import UUID


class Role(StrEnum):
    """Organisation membership roles."""

    OWNER = "owner"
    RESEARCHER = "researcher"
    VIEWER = "viewer"

    @property
    def can_write(self) -> bool:
        """Whether the role may create or modify scientific resources."""
        return self in {Role.OWNER, Role.RESEARCHER}

    @property
    def can_manage_members(self) -> bool:
        """Whether the role may add, change, or remove members."""
        return self is Role.OWNER


@dataclass(frozen=True, slots=True)
class User:
    """A local user mapped to an external identity provider subject."""

    id: UUID
    identity_provider: str
    external_subject: str
    email: str | None
    display_name: str | None
    disabled_at: datetime | None

    @property
    def is_active(self) -> bool:
        return self.disabled_at is None


@dataclass(frozen=True, slots=True)
class Organisation:
    """A tenant boundary."""

    id: UUID
    name: str
    slug: str | None
    status: str
    created_at: datetime
    updated_at: datetime

    @property
    def is_active(self) -> bool:
        return self.status == "active"


@dataclass(frozen=True, slots=True)
class Membership:
    """A user's role within an organisation."""

    id: UUID
    organisation_id: UUID
    user_id: UUID
    role: Role
    created_at: datetime
    updated_at: datetime


@dataclass(frozen=True, slots=True)
class MembershipView:
    """A membership joined with the member's profile, for listing and responses."""

    membership: Membership
    display_name: str | None
    email: str | None


@dataclass(frozen=True, slots=True)
class OrganisationContext:
    """A verified tenant context: the caller, organisation, and their membership."""

    user: User
    organisation: Organisation
    membership: Membership

    @property
    def role(self) -> Role:
        return self.membership.role
