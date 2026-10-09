"""Study operations: tenant scoping, role checks, lifecycle rules, and audit."""

from __future__ import annotations

from collections.abc import Mapping
from typing import Any
from uuid import UUID

from app.core.exceptions import (
    ConflictError,
    ErrorDetail,
    NotFoundError,
    UnprocessableEntityError,
)
from app.core.pagination import Cursor, decode_cursor, encode_cursor
from app.db.pool import Database
from app.models.identity import OrganisationContext, Role
from app.models.study import Study, StudyStatus
from app.repositories import audit
from app.repositories import studies as studies_repository
from app.services import tenancy

_WRITE_ROLES = frozenset({Role.OWNER, Role.RESEARCHER})

_UPDATABLE_FIELDS = ("name", "description")


def _decode(cursor: str | None) -> Cursor | None:
    return decode_cursor(cursor) if cursor is not None else None


async def create_study(
    database: Database,
    *,
    context: OrganisationContext,
    name: str,
    description: str | None,
    request_id: str | None = None,
) -> Study:
    """Create a study owned by the caller's organisation."""
    tenancy.require_role(context, _WRITE_ROLES)
    async with database.transaction() as connection:
        study = await studies_repository.create_study(
            connection,
            organisation_id=context.organisation.id,
            name=name,
            description=description,
            created_by=context.user.id,
        )
        await audit.record_audit_event(
            connection,
            action="study.created",
            resource_type="study",
            organisation_id=context.organisation.id,
            actor_user_id=context.user.id,
            resource_id=study.id,
            request_id=request_id,
            metadata={"name": name},
        )
    return study


async def list_studies(
    database: Database,
    *,
    context: OrganisationContext,
    limit: int,
    cursor: str | None = None,
    search: str | None = None,
    status: StudyStatus | None = None,
) -> tuple[list[Study], str | None, bool]:
    """Return one page of studies plus ``(next_cursor, has_more)``.

    The repository fetches one extra row to distinguish a full page from the
    final page without a second count query.
    """
    async with database.connection() as connection:
        items = await studies_repository.list_studies(
            connection,
            organisation_id=context.organisation.id,
            limit=limit + 1,
            cursor=_decode(cursor),
            status=status,
            search=search,
        )
    has_more = len(items) > limit
    page = items[:limit]
    next_cursor = (
        encode_cursor(created_at=page[-1].created_at, id=page[-1].id) if has_more else None
    )
    return page, next_cursor, has_more


async def get_study(
    database: Database,
    *,
    context: OrganisationContext,
    study_id: UUID,
) -> Study:
    """Read a study, hiding other tenants behind a not-found error."""
    async with database.connection() as connection:
        study = await studies_repository.get_study(
            connection,
            organisation_id=context.organisation.id,
            study_id=study_id,
        )
    if study is None:
        raise NotFoundError("The requested study does not exist.")
    return study


def _normalise_changes(changes: Mapping[str, Any]) -> dict[str, Any]:
    """Validate a partial update; ``name`` may never be set to null."""
    data: dict[str, Any] = {}
    for field in _UPDATABLE_FIELDS:
        if field not in changes:
            continue
        value = changes[field]
        if field == "name" and value is None:
            raise UnprocessableEntityError(
                "The request contains invalid fields.",
                details=[
                    ErrorDetail(
                        "The study name cannot be null.",
                        code="null_not_allowed",
                        field="name",
                    )
                ],
            )
        data[field] = value
    return data


def _changed_fields(before: Study, after: Study) -> dict[str, Any]:
    changed: dict[str, Any] = {}
    for field in _UPDATABLE_FIELDS:
        if getattr(before, field) != getattr(after, field):
            changed[field] = getattr(after, field)
    return changed


async def update_study(
    database: Database,
    *,
    context: OrganisationContext,
    study_id: UUID,
    changes: Mapping[str, Any],
    request_id: str | None = None,
) -> Study:
    """Apply a partial update; archived studies are read-only."""
    tenancy.require_role(context, _WRITE_ROLES)
    async with database.transaction() as connection:
        current = await studies_repository.get_study(
            connection,
            organisation_id=context.organisation.id,
            study_id=study_id,
        )
        if current is None:
            raise NotFoundError("The requested study does not exist.")
        if current.is_archived:
            raise ConflictError(
                "Archived studies are read-only and cannot be modified.",
                code="archived_study",
            )
        updated = await studies_repository.update_study(
            connection,
            organisation_id=context.organisation.id,
            study_id=study_id,
            changes=_normalise_changes(changes),
        )
        assert updated is not None
        changed = _changed_fields(current, updated)
        if changed:
            await audit.record_audit_event(
                connection,
                action="study.updated",
                resource_type="study",
                organisation_id=context.organisation.id,
                actor_user_id=context.user.id,
                resource_id=study_id,
                request_id=request_id,
                metadata=changed,
            )
    return updated


async def archive_study(
    database: Database,
    *,
    context: OrganisationContext,
    study_id: UUID,
    request_id: str | None = None,
) -> Study:
    """Archive a study; archiving an already-archived study is idempotent."""
    tenancy.require_role(context, _WRITE_ROLES)
    async with database.transaction() as connection:
        archived = await studies_repository.archive_study(
            connection,
            organisation_id=context.organisation.id,
            study_id=study_id,
        )
        if archived is not None:
            await audit.record_audit_event(
                connection,
                action="study.archived",
                resource_type="study",
                organisation_id=context.organisation.id,
                actor_user_id=context.user.id,
                resource_id=study_id,
                request_id=request_id,
            )
            return archived
        existing = await studies_repository.get_study(
            connection,
            organisation_id=context.organisation.id,
            study_id=study_id,
        )
    if existing is None:
        raise NotFoundError("The requested study does not exist.")
    return existing
