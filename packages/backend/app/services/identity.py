"""Identity operations: just-in-time user provisioning."""

from __future__ import annotations

from app.db.pool import Database
from app.models.identity import Role, User
from app.repositories import audit, memberships, organisations, users


async def provision_user(
    database: Database,
    *,
    identity_provider: str,
    external_subject: str,
    display_name: str | None = None,
    email: str | None = None,
) -> User:
    """Create or refresh the local user record for an authenticated subject."""
    async with database.transaction() as connection:
        return await users.upsert_user(
            connection,
            identity_provider=identity_provider,
            external_subject=external_subject,
            display_name=display_name,
            email=email,
        )


async def ensure_development_organisation(
    database: Database,
    *,
    user: User,
    name: str,
) -> None:
    """Give a development user an owner membership when they have none.

    Development sign-in is the only caller. It seeds a personal organisation so
    the workspace is usable locally without an administrative provisioning step
    (FR-006). The operation is idempotent: a user who already belongs to any
    organisation is left untouched.
    """
    async with database.transaction() as connection:
        existing = await organisations.list_organisations_for_user(connection, user.id)
        if existing or not user.is_active:
            return
        organisation = await organisations.create_organisation(connection, name=name)
        membership = await memberships.create_membership(
            connection,
            organisation_id=organisation.id,
            user_id=user.id,
            role=Role.OWNER,
        )
        await audit.record_audit_event(
            connection,
            action="organisation.created",
            resource_type="organisation",
            organisation_id=organisation.id,
            actor_user_id=user.id,
            resource_id=organisation.id,
            metadata={"name": name, "seeded": True},
        )
        await audit.record_audit_event(
            connection,
            action="membership.created",
            resource_type="membership",
            organisation_id=organisation.id,
            actor_user_id=user.id,
            resource_id=membership.id,
            metadata={
                "role": Role.OWNER.value,
                "member_user_id": str(user.id),
                "seeded": True,
            },
        )
