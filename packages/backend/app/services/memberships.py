"""Membership operations enforcing the last-owner invariant and audit trail."""

from __future__ import annotations

from uuid import UUID

from app.core.exceptions import ConflictError, NotFoundError
from app.db.pool import Database
from app.models.identity import MembershipView, Organisation, Role, User
from app.repositories import audit, memberships, users

_LAST_OWNER_MESSAGE = "An organisation must always retain at least one owner."


async def list_organisation_members(
    database: Database, *, organisation_id: UUID
) -> list[MembershipView]:
    async with database.connection() as connection:
        return await memberships.list_memberships(connection, organisation_id)


async def add_member(
    database: Database,
    *,
    actor: User,
    organisation: Organisation,
    identity_provider: str,
    external_subject: str,
    role: Role,
    display_name: str | None = None,
    email: str | None = None,
    request_id: str | None = None,
) -> MembershipView:
    """Add a member, provisioning their user record when necessary."""
    async with database.transaction() as connection:
        target = await users.get_user_by_identity(
            connection,
            identity_provider=identity_provider,
            external_subject=external_subject,
        )
        if target is not None and not target.is_active:
            raise ConflictError("This user account is disabled.", code="user_disabled")
        if target is None:
            target = await users.upsert_user(
                connection,
                identity_provider=identity_provider,
                external_subject=external_subject,
                display_name=display_name,
                email=email,
            )
        membership = await memberships.create_membership(
            connection,
            organisation_id=organisation.id,
            user_id=target.id,
            role=role,
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
                "role": role.value,
                "member_user_id": str(target.id),
                "member_identity_provider": identity_provider,
                "member_external_subject": external_subject,
            },
        )
    return MembershipView(
        membership=membership,
        display_name=target.display_name,
        email=target.email,
    )


async def change_member_role(
    database: Database,
    *,
    actor: User,
    organisation: Organisation,
    membership_id: UUID,
    role: Role,
    request_id: str | None = None,
) -> MembershipView:
    """Change a member's role, refusing to demote the last owner."""
    async with database.transaction() as connection:
        await memberships.lock_memberships(connection, organisation.id)
        membership = await memberships.get_membership_by_id(
            connection, organisation_id=organisation.id, membership_id=membership_id
        )
        if membership is None:
            raise NotFoundError("The requested membership does not exist.")
        if membership.role is role:
            target = await users.get_user(connection, membership.user_id)
            return MembershipView(
                membership=membership,
                display_name=target.display_name if target is not None else None,
                email=target.email if target is not None else None,
            )
        if (
            membership.role is Role.OWNER
            and role is not Role.OWNER
            and await memberships.count_owners(connection, organisation.id) <= 1
        ):
            raise ConflictError(_LAST_OWNER_MESSAGE, code="last_owner")
        updated = await memberships.update_membership_role(
            connection,
            organisation_id=organisation.id,
            membership_id=membership_id,
            role=role,
        )
        assert updated is not None
        await audit.record_audit_event(
            connection,
            action="membership.role_changed",
            resource_type="membership",
            organisation_id=organisation.id,
            actor_user_id=actor.id,
            resource_id=membership.id,
            request_id=request_id,
            metadata={
                "previous_role": membership.role.value,
                "role": role.value,
                "member_user_id": str(membership.user_id),
            },
        )
        target = await users.get_user(connection, membership.user_id)
    return MembershipView(
        membership=updated,
        display_name=target.display_name if target is not None else None,
        email=target.email if target is not None else None,
    )


async def remove_member(
    database: Database,
    *,
    actor: User,
    organisation: Organisation,
    membership_id: UUID,
    request_id: str | None = None,
) -> None:
    """Remove a member, refusing to remove the last owner."""
    async with database.transaction() as connection:
        await memberships.lock_memberships(connection, organisation.id)
        membership = await memberships.get_membership_by_id(
            connection, organisation_id=organisation.id, membership_id=membership_id
        )
        if membership is None:
            raise NotFoundError("The requested membership does not exist.")
        if membership.role is Role.OWNER and (
            await memberships.count_owners(connection, organisation.id) <= 1
        ):
            raise ConflictError(_LAST_OWNER_MESSAGE, code="last_owner")
        await memberships.delete_membership(
            connection,
            organisation_id=organisation.id,
            membership_id=membership_id,
        )
        await audit.record_audit_event(
            connection,
            action="membership.removed",
            resource_type="membership",
            organisation_id=organisation.id,
            actor_user_id=actor.id,
            resource_id=membership.id,
            request_id=request_id,
            metadata={
                "role": membership.role.value,
                "member_user_id": str(membership.user_id),
            },
        )
