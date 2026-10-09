"""Current-user identity endpoint."""

from __future__ import annotations

from fastapi import APIRouter, Depends

from app.api.dependencies import get_current_user, get_database
from app.db.pool import Database
from app.models.identity import User
from app.schemas.identity import CurrentUserResponse, MembershipSummary
from app.services import tenancy

router = APIRouter(tags=["identity"])


@router.get(
    "/me",
    response_model=CurrentUserResponse,
    summary="Describe the authenticated user",
    description=(
        "Returns the caller's profile and organisation memberships. Provider "
        "internals and access tokens are never included."
    ),
)
async def read_current_user(
    user: User = Depends(get_current_user),
    database: Database = Depends(get_database),
) -> CurrentUserResponse:
    organisations = await tenancy.list_user_organisations(database, user_id=user.id)
    return CurrentUserResponse(
        id=user.id,
        display_name=user.display_name,
        email=user.email,
        memberships=[
            MembershipSummary(
                membership_id=membership.id,
                organisation_id=organisation.id,
                organisation_name=organisation.name,
                role=membership.role,
            )
            for organisation, membership in organisations
        ],
    )
