"""Organisation and membership endpoints."""

from __future__ import annotations

from uuid import UUID

from fastapi import APIRouter, Depends, Request, Response, status

from app.api.dependencies import get_current_user, get_database, get_organisation_context
from app.db.pool import Database
from app.models.identity import MembershipView, OrganisationContext, Role, User
from app.schemas.identity import (
    MembershipCreateRequest,
    MembershipListResponse,
    MembershipRead,
    MembershipRoleUpdateRequest,
    OrganisationCreateRequest,
    OrganisationDetailResponse,
    OrganisationListResponse,
    OrganisationSummary,
)
from app.services import memberships as memberships_service
from app.services import tenancy

router = APIRouter(tags=["organisations"])


def _membership_read(view: MembershipView) -> MembershipRead:
    membership = view.membership
    return MembershipRead(
        id=membership.id,
        user_id=membership.user_id,
        display_name=view.display_name,
        email=view.email,
        role=membership.role,
        created_at=membership.created_at,
        updated_at=membership.updated_at,
    )


def _request_id(request: Request) -> str | None:
    value = getattr(request.state, "request_id", None)
    return value if isinstance(value, str) else None


@router.post(
    "/organisations",
    response_model=OrganisationDetailResponse,
    status_code=status.HTTP_201_CREATED,
    summary="Create an organisation and make the caller an owner",
)
async def create_organisation(
    payload: OrganisationCreateRequest,
    request: Request,
    user: User = Depends(get_current_user),
    database: Database = Depends(get_database),
) -> OrganisationDetailResponse:
    organisation, membership = await tenancy.create_organisation(
        database,
        actor=user,
        name=payload.name,
        slug=payload.slug,
        request_id=_request_id(request),
    )
    return OrganisationDetailResponse(
        id=organisation.id,
        name=organisation.name,
        slug=organisation.slug,
        status=organisation.status,
        role=membership.role,
        created_at=organisation.created_at,
        updated_at=organisation.updated_at,
    )


@router.get(
    "/organisations",
    response_model=OrganisationListResponse,
    summary="List organisations accessible to the caller",
)
async def list_organisations(
    user: User = Depends(get_current_user),
    database: Database = Depends(get_database),
) -> OrganisationListResponse:
    organisations = await tenancy.list_user_organisations(database, user_id=user.id)
    return OrganisationListResponse(
        items=[
            OrganisationSummary(
                id=organisation.id,
                name=organisation.name,
                slug=organisation.slug,
                status=organisation.status,
                role=membership.role,
            )
            for organisation, membership in organisations
        ]
    )


@router.get(
    "/organisations/{organisation_id}",
    response_model=OrganisationDetailResponse,
    summary="Read an organisation the caller belongs to",
)
async def read_organisation(
    context: OrganisationContext = Depends(get_organisation_context),
) -> OrganisationDetailResponse:
    organisation = context.organisation
    return OrganisationDetailResponse(
        id=organisation.id,
        name=organisation.name,
        slug=organisation.slug,
        status=organisation.status,
        role=context.role,
        created_at=organisation.created_at,
        updated_at=organisation.updated_at,
    )


@router.get(
    "/organisations/{organisation_id}/memberships",
    response_model=MembershipListResponse,
    summary="List organisation members",
)
async def list_memberships(
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
) -> MembershipListResponse:
    views = await memberships_service.list_organisation_members(
        database, organisation_id=context.organisation.id
    )
    return MembershipListResponse(items=[_membership_read(view) for view in views])


@router.post(
    "/organisations/{organisation_id}/memberships",
    response_model=MembershipRead,
    status_code=status.HTTP_201_CREATED,
    summary="Add an organisation member (owner only)",
)
async def add_membership(
    payload: MembershipCreateRequest,
    request: Request,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
) -> MembershipRead:
    tenancy.require_role(context, {Role.OWNER})
    view = await memberships_service.add_member(
        database,
        actor=context.user,
        organisation=context.organisation,
        identity_provider=payload.identity_provider,
        external_subject=payload.subject,
        role=payload.role,
        display_name=payload.display_name,
        email=payload.email,
        request_id=_request_id(request),
    )
    return _membership_read(view)


@router.patch(
    "/organisations/{organisation_id}/memberships/{membership_id}",
    response_model=MembershipRead,
    summary="Change a member's role (owner only)",
)
async def change_membership_role(
    membership_id: UUID,
    payload: MembershipRoleUpdateRequest,
    request: Request,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
) -> MembershipRead:
    tenancy.require_role(context, {Role.OWNER})
    view = await memberships_service.change_member_role(
        database,
        actor=context.user,
        organisation=context.organisation,
        membership_id=membership_id,
        role=payload.role,
        request_id=_request_id(request),
    )
    return _membership_read(view)


@router.delete(
    "/organisations/{organisation_id}/memberships/{membership_id}",
    status_code=status.HTTP_204_NO_CONTENT,
    summary="Remove an organisation member (owner only)",
)
async def remove_membership(
    membership_id: UUID,
    request: Request,
    context: OrganisationContext = Depends(get_organisation_context),
    database: Database = Depends(get_database),
) -> Response:
    tenancy.require_role(context, {Role.OWNER})
    await memberships_service.remove_member(
        database,
        actor=context.user,
        organisation=context.organisation,
        membership_id=membership_id,
        request_id=_request_id(request),
    )
    return Response(status_code=status.HTTP_204_NO_CONTENT)
