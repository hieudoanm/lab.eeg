"""Dataset operations: tenant scoping, role checks, lifecycle rules, and audit."""

from __future__ import annotations

from typing import Any
from uuid import UUID

from app.core.exceptions import ConflictError, NotFoundError
from app.core.pagination import decode_cursor, encode_cursor
from app.db.pool import Database
from app.models.dataset import Dataset, DatasetFormat
from app.models.identity import OrganisationContext, Role
from app.repositories import audit
from app.repositories import datasets as datasets_repository
from app.repositories import studies as studies_repository
from app.services import tenancy

_WRITE_ROLES = frozenset({Role.OWNER, Role.RESEARCHER})


async def register_dataset(
    database: Database,
    *,
    context: OrganisationContext,
    study_id: UUID,
    name: str,
    description: str | None,
    dataset_format: DatasetFormat,
    source_description: str | None,
    metadata: dict[str, Any],
    request_id: str | None = None,
) -> Dataset:
    """Register dataset metadata in an existing, active study.

    The dataset starts in ``pending_upload``; the backend owns its identifier
    and storage key, and the client cannot mark it ready.
    """
    tenancy.require_role(context, _WRITE_ROLES)
    async with database.transaction() as connection:
        study = await studies_repository.get_study(
            connection,
            organisation_id=context.organisation.id,
            study_id=study_id,
        )
        if study is None:
            raise NotFoundError("The requested study does not exist.")
        if study.is_archived:
            raise ConflictError(
                "Datasets cannot be registered in archived studies.",
                code="archived_study",
            )
        dataset = await datasets_repository.create_dataset(
            connection,
            organisation_id=context.organisation.id,
            study_id=study_id,
            name=name,
            description=description,
            dataset_format=dataset_format,
            source_description=source_description,
            metadata=metadata,
            uploaded_by=context.user.id,
        )
        await audit.record_audit_event(
            connection,
            action="dataset.registered",
            resource_type="dataset",
            organisation_id=context.organisation.id,
            actor_user_id=context.user.id,
            resource_id=dataset.id,
            request_id=request_id,
            metadata={
                "study_id": str(study_id),
                "name": name,
                "format": dataset_format.value,
            },
        )
    return dataset


async def list_datasets(
    database: Database,
    *,
    context: OrganisationContext,
    study_id: UUID,
    limit: int,
    cursor: str | None = None,
) -> tuple[list[Dataset], str | None, bool]:
    """Return one page of datasets for a study the caller can access."""
    async with database.connection() as connection:
        study = await studies_repository.get_study(
            connection,
            organisation_id=context.organisation.id,
            study_id=study_id,
        )
        if study is None:
            raise NotFoundError("The requested study does not exist.")
        items = await datasets_repository.list_datasets(
            connection,
            organisation_id=context.organisation.id,
            study_id=study_id,
            limit=limit + 1,
            cursor=decode_cursor(cursor) if cursor is not None else None,
        )
    has_more = len(items) > limit
    page = items[:limit]
    next_cursor = (
        encode_cursor(created_at=page[-1].created_at, id=page[-1].id) if has_more else None
    )
    return page, next_cursor, has_more


async def get_dataset(
    database: Database,
    *,
    context: OrganisationContext,
    dataset_id: UUID,
) -> Dataset:
    """Read a dataset, hiding other tenants behind a not-found error."""
    async with database.connection() as connection:
        dataset = await datasets_repository.get_dataset(
            connection,
            organisation_id=context.organisation.id,
            dataset_id=dataset_id,
        )
    if dataset is None:
        raise NotFoundError("The requested dataset does not exist.")
    return dataset
