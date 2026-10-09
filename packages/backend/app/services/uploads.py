"""Upload operations: session lifecycle, content handling, and validation.

Phase 5 validates uploads synchronously on finalisation so the full
registration → upload → validate → ready workflow works end to end without the
background worker. The background pipeline (Phase 6) may move validation into a
worker without changing the client contract.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from typing import Any, NoReturn
from uuid import UUID, uuid4

from app.core.config import Settings
from app.core.exceptions import (
    ConflictError,
    NotFoundError,
    PayloadTooLargeError,
    UnprocessableEntityError,
)
from app.db.pool import Database
from app.models.dataset import Dataset, DatasetStatus
from app.models.identity import OrganisationContext, Role
from app.models.upload_session import UploadSession, UploadSessionStatus
from app.repositories import audit
from app.repositories import datasets as datasets_repository
from app.repositories import upload_sessions as sessions_repository
from app.schemas.upload import UploadManifest, UploadSessionCreateRequest
from app.scientific import ChannelManifest, RecordingInvalidError, validate_recording
from app.services import datasets as datasets_service
from app.services import tenancy
from app.storage.base import Storage

_WRITE_ROLES = frozenset({Role.OWNER, Role.RESEARCHER})
_UPLOADABLE_DATASET_STATUSES = frozenset(
    {DatasetStatus.PENDING_UPLOAD, DatasetStatus.REJECTED, DatasetStatus.FAILED}
)


@dataclass(frozen=True, slots=True)
class FinaliseResult:
    """Outcome of a finalisation request, including the resulting dataset."""

    dataset: Dataset
    session: UploadSession


@dataclass(frozen=True, slots=True)
class CleanupReport:
    """Summary of an abandoned-upload cleanup run."""

    expired_sessions: int
    objects_deleted: int
    objects_failed: int
    batches: int


@dataclass(frozen=True, slots=True)
class _ValidatedRecording:
    """Recording properties verified against its manifest."""

    sampling_frequency_hz: float
    channel_count: int
    sample_count: int
    duration_seconds: float
    channel_names: list[str]
    channel_units: list[str] | None
    metadata: dict[str, Any]


class _RecordingRejected(Exception):
    """Internal signal that a recording failed validation."""

    def __init__(self, message: str) -> None:
        self.message = message
        super().__init__(message)


def _object_key(organisation_id: UUID, dataset_id: UUID) -> str:
    return f"organisations/{organisation_id}/datasets/{dataset_id}/{uuid4()}.npy"


def _now() -> datetime:
    return datetime.now(UTC)


async def create_upload_session(
    database: Database,
    storage: Storage,
    settings: Settings,
    *,
    context: OrganisationContext,
    dataset_id: UUID,
    payload: UploadSessionCreateRequest,
    request_id: str | None = None,
) -> UploadSession:
    """Create an upload session for an uploadable, tenant-owned dataset."""
    tenancy.require_role(context, _WRITE_ROLES)
    async with database.transaction() as connection:
        dataset = await datasets_repository.get_dataset(
            connection,
            organisation_id=context.organisation.id,
            dataset_id=dataset_id,
        )
        if dataset is None:
            raise NotFoundError("The requested dataset does not exist.")
        if dataset.status not in _UPLOADABLE_DATASET_STATUSES:
            raise ConflictError(
                "This dataset cannot accept another upload.",
                code="dataset_not_uploadable",
            )

        await sessions_repository.expire_stale_sessions(
            connection,
            organisation_id=context.organisation.id,
            dataset_id=dataset_id,
            now=_now(),
        )

        session = await sessions_repository.create_upload_session(
            connection,
            organisation_id=context.organisation.id,
            dataset_id=dataset_id,
            created_by=context.user.id,
            object_key=_object_key(context.organisation.id, dataset_id),
            expected_size_bytes=payload.file_size_bytes,
            expected_checksum_sha256=payload.expected_checksum_sha256,
            expires_at=_now() + timedelta(seconds=settings.upload_session_ttl_seconds),
            manifest=payload.manifest.model_dump(),
        )
        await audit.record_audit_event(
            connection,
            action="upload_session.created",
            resource_type="dataset",
            organisation_id=context.organisation.id,
            actor_user_id=context.user.id,
            resource_id=dataset_id,
            request_id=request_id,
            metadata={
                "upload_session_id": str(session.id),
                "expected_size_bytes": payload.file_size_bytes,
            },
        )
    return session


async def store_upload_content(
    database: Database,
    storage: Storage,
    settings: Settings,
    *,
    context: OrganisationContext,
    dataset_id: UUID,
    session_id: UUID,
    body: bytes,
) -> None:
    """Persist the recording bytes for a session into private storage."""
    tenancy.require_role(context, _WRITE_ROLES)
    if len(body) > settings.max_upload_bytes:
        raise PayloadTooLargeError(
            f"The recording exceeds the configured limit of {settings.max_upload_bytes} bytes.",
        )

    async with database.connection() as connection:
        session = await sessions_repository.get_upload_session(
            connection,
            organisation_id=context.organisation.id,
            session_id=session_id,
        )
    if session is None:
        raise NotFoundError("The requested upload session does not exist.")
    if session.dataset_id != dataset_id:
        raise NotFoundError("The requested upload session does not exist.")
    if session.status == UploadSessionStatus.UPLOADED:
        return
    if session.status not in {UploadSessionStatus.CREATED, UploadSessionStatus.UPLOADING}:
        if session.is_expired:
            raise ConflictError(
                "This upload session has expired.",
                code="upload_session_expired",
            )
        raise ConflictError(
            "This upload session cannot accept content right now.",
            code="upload_session_conflict",
        )
    if session.is_expired:
        raise ConflictError(
            "This upload session has expired.",
            code="upload_session_expired",
        )

    await storage.write_bytes(_require_object_key(session), body)

    async with database.transaction() as connection:
        updated = await sessions_repository.mark_uploaded(
            connection,
            organisation_id=context.organisation.id,
            session_id=session.id,
        )
        if updated is None:
            raise ConflictError(
                "This upload session is already finalised.",
                code="upload_session_conflict",
            )


async def finalise_upload(
    database: Database,
    storage: Storage,
    settings: Settings,
    *,
    context: OrganisationContext,
    dataset_id: UUID,
    session_id: UUID,
    request_id: str | None = None,
) -> FinaliseResult:
    """Validate the stored recording and complete the dataset lifecycle.

    Returns early (idempotently) when the session was already completed. Any
    validation failure transitions the dataset to ``rejected`` and ends the
    session as ``failed`` before raising a safe client error.
    """
    tenancy.require_role(context, _WRITE_ROLES)

    session = await _get_session(database, context, session_id)
    if session.dataset_id != dataset_id:
        raise NotFoundError("The requested upload session does not exist.")
    if session.status == UploadSessionStatus.COMPLETED:
        dataset = await datasets_service.get_dataset(
            database, context=context, dataset_id=session.dataset_id
        )
        return FinaliseResult(dataset=dataset, session=session)
    if session.is_expired:
        raise ConflictError(
            "This upload session has expired.",
            code="upload_session_expired",
        )
    if session.status != UploadSessionStatus.UPLOADED:
        raise ConflictError(
            "The recording has not been uploaded yet.",
            code="upload_content_missing",
        )

    async with database.transaction() as connection:
        claimed = await sessions_repository.mark_finalising(
            connection,
            organisation_id=context.organisation.id,
            session_id=session.id,
        )
        if claimed is None:
            raise ConflictError(
                "This upload is already being finalised.",
                code="finalisation_in_progress",
            )

    object_key = _require_object_key(session)
    try:
        validated = await _verify_and_validate(storage, settings, object_key, session)
        failure: str | None = None
    except _RecordingRejected as exc:
        failure = exc.message

    if failure is not None:
        async with database.transaction() as connection:
            await datasets_repository.mark_dataset_rejected(
                connection,
                organisation_id=context.organisation.id,
                dataset_id=session.dataset_id,
            )
            await sessions_repository.fail_upload(
                connection,
                organisation_id=context.organisation.id,
                session_id=session.id,
            )
            await audit.record_audit_event(
                connection,
                action="dataset.validation_failed",
                resource_type="dataset",
                organisation_id=context.organisation.id,
                actor_user_id=context.user.id,
                resource_id=session.dataset_id,
                request_id=request_id,
                metadata={
                    "upload_session_id": str(session.id),
                    "reason": failure,
                },
            )
        raise UnprocessableEntityError(failure, code="invalid_recording")

    file_size = await storage.size(object_key)
    checksum = await storage.hash_sha256(object_key)
    existing = await datasets_service.get_dataset(
        database, context=context, dataset_id=session.dataset_id
    )
    metadata = {**existing.metadata, **validated.metadata}

    async with database.transaction() as connection:
        updated = await datasets_repository.mark_dataset_validated(
            connection,
            organisation_id=context.organisation.id,
            dataset_id=session.dataset_id,
            storage_key=object_key,
            checksum_sha256=checksum,
            file_size_bytes=file_size,
            sampling_frequency_hz=validated.sampling_frequency_hz,
            channel_count=validated.channel_count,
            sample_count=validated.sample_count,
            duration_seconds=validated.duration_seconds,
            channel_names=validated.channel_names,
            channel_units=validated.channel_units,
            metadata=metadata,
        )
        if updated is None:
            dataset = await datasets_service.get_dataset(
                database, context=context, dataset_id=session.dataset_id
            )
        else:
            dataset = updated
        completed = await sessions_repository.complete_upload(
            connection,
            organisation_id=context.organisation.id,
            session_id=session.id,
            finalised_at=_now(),
        )
        await audit.record_audit_event(
            connection,
            action="dataset.uploaded",
            resource_type="dataset",
            organisation_id=context.organisation.id,
            actor_user_id=context.user.id,
            resource_id=session.dataset_id,
            request_id=request_id,
            metadata={
                "upload_session_id": str(session.id),
                "checksum_sha256": checksum,
                "file_size_bytes": file_size,
            },
        )
    return FinaliseResult(dataset=dataset, session=completed or session)


async def cleanup_abandoned_uploads(
    database: Database,
    storage: Storage,
    *,
    now: datetime | None = None,
    batch_size: int = 100,
    max_batches: int = 50,
) -> CleanupReport:
    """Expire abandoned upload sessions and delete their stored objects.

    This is an on-demand maintenance operation: scheduling belongs to the
    reliability phase. It runs in bounded batches so it never holds a database
    transaction across object-storage work (AGENTS.md §4.3).

    Each batch is claimed and expired in a short transaction, then its objects
    are deleted best-effort outside any transaction. A missing object counts as
    nothing to do and a failed deletion is recorded without aborting the run;
    expiring the session always takes priority so a transient storage error
    cannot leave a session stuck indefinitely or block other batches.
    """
    deadline = now or _now()
    if batch_size < 1:
        raise ValueError("batch_size must be at least 1")
    if max_batches < 1:
        raise ValueError("max_batches must be at least 1")

    expired_sessions = 0
    objects_deleted = 0
    objects_failed = 0
    batches = 0

    while batches < max_batches:
        async with database.transaction() as connection:
            claimed = await sessions_repository.claim_expired_sessions(
                connection,
                now=deadline,
                limit=batch_size,
            )
            if not claimed:
                break
            await sessions_repository.mark_sessions_expired(
                connection,
                session_ids=[session.id for session in claimed],
            )
        batches += 1
        expired_sessions += len(claimed)

        for session in claimed:
            if session.object_key is None:
                continue
            try:
                if await storage.exists(session.object_key):
                    await storage.delete(session.object_key)
                    objects_deleted += 1
            except Exception:
                # A single object must never abort cleanup of the rest.
                objects_failed += 1

        if len(claimed) < batch_size:
            break

    return CleanupReport(
        expired_sessions=expired_sessions,
        objects_deleted=objects_deleted,
        objects_failed=objects_failed,
        batches=batches,
    )


async def _get_session(
    database: Database,
    context: OrganisationContext,
    session_id: UUID,
) -> UploadSession:
    async with database.connection() as connection:
        session = await sessions_repository.get_upload_session(
            connection,
            organisation_id=context.organisation.id,
            session_id=session_id,
        )
    if session is None:
        raise NotFoundError("The requested upload session does not exist.")
    return session


def _require_object_key(session: UploadSession) -> str:
    if session.object_key is None:
        raise ConflictError(
            "This upload session has no storage target.",
            code="upload_session_conflict",
        )
    return session.object_key


def _reject(message: str) -> NoReturn:
    """Raise the internal rejection signal (typed for callers)."""
    raise _RecordingRejected(message)


async def _verify_and_validate(
    storage: Storage,
    settings: Settings,
    object_key: str,
    session: UploadSession,
) -> _ValidatedRecording:
    """Check the stored object and validate the recording.

    Raises ``_RecordingRejected`` with a safe message on the first violation.
    """
    if not await storage.exists(object_key):
        _reject("The uploaded recording could not be found.")

    actual_size = await storage.size(object_key)
    if actual_size > settings.max_upload_bytes:
        _reject(
            f"The recording is {actual_size} bytes; the limit is {settings.max_upload_bytes} bytes."
        )
    if session.expected_size_bytes is not None and actual_size != session.expected_size_bytes:
        _reject("The recording size does not match the declared file size.")
    if session.expected_checksum_sha256 is not None:
        actual_checksum = await storage.hash_sha256(object_key)
        if actual_checksum != session.expected_checksum_sha256:
            _reject("The recording checksum does not match the expected value.")

    try:
        manifest = _channel_manifest(session)
        path = storage.local_path(object_key)
        if path is None:
            _reject("This storage backend cannot validate recordings in place.")

        # Validation reads only the .npy header (memory-mapped), so a large but
        # misconfigured recording does not get materialised into API memory.
        recording = validate_recording(
            path,
            manifest=manifest,
            max_channels=settings.max_channels,
            max_sample_count=settings.max_sample_count,
        )
    except RecordingInvalidError as exc:
        _reject(str(exc))
    return _ValidatedRecording(
        sampling_frequency_hz=recording.sampling_frequency_hz,
        channel_count=recording.channel_count,
        sample_count=recording.sample_count,
        duration_seconds=recording.duration_seconds,
        channel_names=manifest.channel_names,
        channel_units=manifest.channel_units,
        metadata={"recording_dtype": recording.dtype_name},
    )


def _channel_manifest(session: UploadSession) -> ChannelManifest:
    manifest = UploadManifest.model_validate(session.manifest)
    units = manifest.channel_units
    if units is not None and len(units) != len(manifest.channel_names):
        raise RecordingInvalidError(
            "The channel unit metadata does not match the number of channels."
        )
    return ChannelManifest(
        sampling_frequency_hz=manifest.sampling_frequency_hz,
        channel_names=manifest.channel_names,
        channel_units=units,
    )


__all__ = [
    "CleanupReport",
    "FinaliseResult",
    "cleanup_abandoned_uploads",
    "create_upload_session",
    "finalise_upload",
    "store_upload_content",
]
