"""Tenant-context resolution and role checks."""

from __future__ import annotations

from collections.abc import Collection
from uuid import UUID

from app.core.exceptions import ForbiddenError, NotFoundError
from app.db.pool import Database
from app.models.identity import Membership, Organisation, OrganisationContext, Role, User
from app.repositories import audit, memberships, organisations


async def create_organisation(
    database: Database,
    *,
    actor: User,
    name: str,
    slug: str | None = None,
    request_id: str | None = None,
) -> tuple[Organisation, Membership]:
    """Create an organisation and make the actor its owner."""
    async with database.transaction() as connection:
        organisation = await organisations.create_organisation(connection, name=name, slug=slug)
        membership = await memberships.create_membership(
            connection,
            organisation_id=organisation.id,
            user_id=actor.id,
            role=Role.OWNER,
        )
        await audit.record_audit_event(
            connection,
            action="organisation.created",
            resource_type="organisation",
            organisation_id=organisation.id,
            actor_user_id=actor.id,
            resource_id=organisation.id,
            request_id=request_id,
            metadata={"name": name, "slug": slug},
        )
        await audit.record_audit_event(
            connection,
            action="membership.created",
            resource_type="membership",
            organisation_id=organisation.id,
            actor_user_id=actor.id,
            resource_id=membership.id,
            request_id=request_id,
            metadata={
                "role": Role.OWNER.value,
                "member_user_id": str(actor.id),
            },
        )
    return organisation, membership


async def list_user_organisations(
    database: Database,
    *,
    user_id: UUID,
) -> list[tuple[Organisation, Membership]]:
    """Return every organisation the user belongs to, with their membership.

    The list is derived only from the authenticated user's membership records,
    never from a client-supplied identifier.
    """
    async with database.connection() as connection:
        return await organisations.list_organisations_for_user(connection, user_id)


async def load_organisation_context(
    database: Database,
    *,
    user: User,
    organisation_id: UUID,
) -> OrganisationContext:
    """Resolve the caller's membership in an organisation.

    A missing organisation and a missing membership both raise ``NotFoundError``
    so that the API never discloses the existence of another tenant's resource.
    """
    async with database.connection() as connection:
        organisation = await organisations.get_organisation(connection, organisation_id)
        if organisation is None:
            raise NotFoundError("The requested organisation does not exist.")
        membership = await memberships.get_membership(
            connection, organisation_id=organisation_id, user_id=user.id
        )
    if membership is None:
        raise NotFoundError("The requested organisation does not exist.")
    return OrganisationContext(user=user, organisation=organisation, membership=membership)


def require_role(context: OrganisationContext, allowed: Collection[Role]) -> None:
    """Raise ``ForbiddenError`` unless the caller holds an allowed role."""
    if context.role not in allowed:
        raise ForbiddenError(
            "Your role in this organisation does not permit this action.",
            code="insufficient_role",
        )
