"""API tests for the upload session and validation endpoints."""

from __future__ import annotations

import hashlib
import io
from typing import Any, cast
from uuid import UUID

import numpy as np
import psycopg
from app.main import create_app
from httpx import AsyncClient

from tests.conftest import (
    bearer,
    create_membership,
    create_organisation,
    create_user,
    issue_token,
    make_settings,
)

STUDIES = "/api/v1/organisations/{organisation_id}/studies"
DATASETS = "/api/v1/organisations/{organisation_id}/studies/{study_id}/datasets"
SESSIONS = "/api/v1/organisations/{organisation_id}/datasets/{dataset_id}/upload-sessions"
FINALISE = (
    "/api/v1/organisations/{organisation_id}/datasets/{dataset_id}"
    "/upload-sessions/{session_id}/finalise"
)
MISSING = "00000000-0000-4000-8000-000000000000"


async def test_upload_put_is_allowed_by_cors(client: AsyncClient) -> None:
    response = await client.options(
        "/api/v1/organisations/00000000-0000-4000-8000-000000000000"
        "/datasets/00000000-0000-4000-8000-000000000000"
        "/upload-sessions/00000000-0000-4000-8000-000000000000/content",
        headers={
            "Origin": "http://localhost:3000",
            "Access-Control-Request-Method": "PUT",
            "Access-Control-Request-Headers": "authorization,content-type",
        },
    )
    assert response.status_code == 200
    assert "PUT" in response.headers["access-control-allow-methods"]


def _seed_member(
    db: psycopg.Connection[dict[str, Any]],
    *,
    subject: str,
    role: str,
    organisation_name: str = "Research Organisation",
) -> tuple[UUID, UUID]:
    user_id = create_user(db, subject=subject)
    organisation_id = create_organisation(db, name=organisation_name)
    create_membership(db, organisation_id=organisation_id, user_id=user_id, role=role)
    return user_id, organisation_id


async def _create_study(
    client: AsyncClient, token: str, organisation_id: UUID, name: str = "Study"
) -> dict[str, Any]:
    response = await client.post(
        STUDIES.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={"name": name},
    )
    assert response.status_code == 201, response.text
    return cast(dict[str, Any], response.json())


async def _register_dataset(
    client: AsyncClient,
    token: str,
    organisation_id: UUID,
    study_id: str,
    name: str = "Session 1",
) -> dict[str, Any]:
    response = await client.post(
        DATASETS.format(organisation_id=organisation_id, study_id=study_id),
        headers=bearer(token),
        json={"name": name},
    )
    assert response.status_code == 201, response.text
    return cast(dict[str, Any], response.json())


def _manifest(
    *,
    frequency: float = 256.0,
    channels: list[str] | None = None,
    units: list[str] | None = None,
) -> dict[str, Any]:
    names = channels or ["F3", "F4", "C3", "C4"]
    body: dict[str, Any] = {"sampling_frequency_hz": frequency, "channel_names": names}
    if units is not None:
        body["channel_units"] = units
    return body


def _npy_bytes(
    *,
    channels: int = 4,
    samples: int = 256,
    dtype: str = "float32",
) -> bytes:
    buffer = io.BytesIO()
    np.save(buffer, np.zeros((channels, samples), dtype=dtype), allow_pickle=False)
    return buffer.getvalue()


async def _create_session(
    client: AsyncClient,
    token: str,
    organisation_id: UUID,
    dataset_id: str,
    data: bytes,
    **extra: object,
) -> dict[str, Any]:
    payload: dict[str, object] = {
        "file_name": "recording.npy",
        "file_size_bytes": len(data),
        "format": "npy",
        "manifest": _manifest(),
        **extra,
    }
    response = await client.post(
        SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset_id),
        headers=bearer(token),
        json=payload,
    )
    assert response.status_code == 201, response.text
    return cast(dict[str, Any], response.json())


async def _upload_and_finalise(
    client: AsyncClient,
    token: str,
    organisation_id: UUID,
    dataset_id: str,
    data: bytes,
    *,
    session: dict[str, Any] | None = None,
) -> dict[str, Any]:
    created = session or await _create_session(client, token, organisation_id, dataset_id, data)
    content = await client.put(
        f"{SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset_id)}"
        f"/{created['upload_session_id']}/content",
        headers=bearer(token),
        content=data,
    )
    assert content.status_code == 204, content.text
    response = await client.post(
        FINALISE.format(
            organisation_id=organisation_id,
            dataset_id=dataset_id,
            session_id=created["upload_session_id"],
        ),
        headers=bearer(token),
    )
    assert response.status_code == 200, response.text
    return cast(dict[str, Any], response.json())


async def test_full_upload_lifecycle_succeeds(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(client, token, organisation_id, study["id"])
    data = _npy_bytes(channels=8, samples=512, dtype="float32")
    units = ["uV"] * 8

    response = await client.post(
        SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=bearer(token),
        json={
            "file_name": "recording.npy",
            "file_size_bytes": len(data),
            "manifest": _manifest(channels=[f"C{i}" for i in range(8)], units=units),
        },
    )
    assert response.status_code == 201, response.text
    session = response.json()
    assert session["status"] == "created"
    assert session["upload_method"] == "backend"
    assert session["expires_at"].endswith("Z")
    assert session["upload_url"].endswith(
        f"/upload-sessions/{session['upload_session_id']}/content"
    )

    content = await client.put(
        f"{SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset['id'])}"
        f"/{session['upload_session_id']}/content",
        headers=bearer(token),
        content=data,
    )
    assert content.status_code == 204, content.text

    before = await client.get(
        f"/api/v1/organisations/{organisation_id}/datasets/{dataset['id']}",
        headers=bearer(token),
    )
    assert before.json()["status"] == "pending_upload"

    finalise = await client.post(
        FINALISE.format(
            organisation_id=organisation_id,
            dataset_id=dataset["id"],
            session_id=session["upload_session_id"],
        ),
        headers=bearer(token),
    )
    assert finalise.status_code == 200, finalise.text
    body = finalise.json()
    assert body["status"] == "completed"
    ready = body["dataset"]
    assert ready["status"] == "ready"
    assert ready["sampling_frequency_hz"] == 256.0
    assert ready["channel_count"] == 8
    assert ready["sample_count"] == 512
    assert ready["duration_seconds"] == 2.0
    assert ready["channel_names"] == [f"C{i}" for i in range(8)]
    assert ready["channel_units"] == units
    assert ready["checksum_sha256"] == hashlib.sha256(data).hexdigest()
    assert ready["validated_at"] is not None
    assert "storage_key" not in ready


async def test_finalise_is_idempotent(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(client, token, organisation_id, study["id"])
    data = _npy_bytes()
    created = await _create_session(client, token, organisation_id, dataset["id"], data)

    url = FINALISE.format(
        organisation_id=organisation_id,
        dataset_id=dataset["id"],
        session_id=created["upload_session_id"],
    )
    content_url = (
        f"{SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset['id'])}"
        f"/{created['upload_session_id']}/content"
    )
    await client.put(content_url, headers=bearer(token), content=data)
    first = await client.post(url, headers=bearer(token))
    assert first.status_code == 200, first.text
    second = await client.post(url, headers=bearer(token))
    assert second.status_code == 200, second.text
    assert second.json()["dataset"]["id"] == first.json()["dataset"]["id"]
    assert second.json()["dataset"]["status"] == "ready"


async def test_viewer_cannot_create_upload_session(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="owner", role="owner")
    token = await issue_token(client, subject="owner")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(client, token, organisation_id, study["id"])
    viewer_id = create_user(db, subject="viewer-upload")
    create_membership(db, organisation_id=organisation_id, user_id=viewer_id, role="viewer")
    viewer_token = await issue_token(client, subject="viewer-upload")
    response = await client.post(
        SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=bearer(viewer_token),
        json={
            "file_name": "rec.npy",
            "file_size_bytes": 10,
            "manifest": _manifest(),
        },
    )
    assert response.status_code == 403
    assert response.json()["error"]["code"] == "insufficient_role"


async def test_upload_session_is_scoped_to_organisation(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, org_a = _seed_member(
        db, subject="researcher-a", role="researcher", organisation_name="Organisation A"
    )
    token_a = await issue_token(client, subject="researcher-a")
    study = await _create_study(client, token_a, org_a)
    dataset = await _register_dataset(client, token_a, org_a, study["id"])
    data = _npy_bytes()
    _, org_b = _seed_member(
        db, subject="researcher-b", role="researcher", organisation_name="Organisation B"
    )
    token_b = await issue_token(client, subject="researcher-b")

    created = await _create_session(client, token_a, org_a, dataset["id"], data)
    mismatched = await client.post(
        SESSIONS.format(organisation_id=org_b, dataset_id=dataset["id"]),
        headers=bearer(token_b),
        json={
            "file_name": "rec.npy",
            "file_size_bytes": len(data),
            "manifest": _manifest(),
        },
    )
    assert mismatched.status_code == 404

    content = await client.put(
        f"{SESSIONS.format(organisation_id=org_a, dataset_id=dataset['id'])}"
        f"/{created['upload_session_id']}/content",
        headers=bearer(token_b),
        content=data,
    )
    assert content.status_code == 404


async def test_upload_session_is_scoped_to_dataset(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(client, token, organisation_id, study["id"], name="Dataset A")
    other_dataset = await _register_dataset(
        client, token, organisation_id, study["id"], name="Dataset B"
    )
    data = _npy_bytes()
    created = await _create_session(client, token, organisation_id, dataset["id"], data)

    wrong_content_url = (
        f"{SESSIONS.format(organisation_id=organisation_id, dataset_id=other_dataset['id'])}"
        f"/{created['upload_session_id']}/content"
    )
    wrong_content = await client.put(wrong_content_url, headers=bearer(token), content=data)
    assert wrong_content.status_code == 404

    content_url = (
        f"{SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset['id'])}"
        f"/{created['upload_session_id']}/content"
    )
    content = await client.put(content_url, headers=bearer(token), content=data)
    assert content.status_code == 204

    wrong_finalise_url = FINALISE.format(
        organisation_id=organisation_id,
        dataset_id=other_dataset["id"],
        session_id=created["upload_session_id"],
    )
    wrong_finalise = await client.post(wrong_finalise_url, headers=bearer(token))
    assert wrong_finalise.status_code == 404

    finalise = await client.post(
        FINALISE.format(
            organisation_id=organisation_id,
            dataset_id=dataset["id"],
            session_id=created["upload_session_id"],
        ),
        headers=bearer(token),
    )
    assert finalise.status_code == 200


async def test_invalid_recording_is_rejected(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(client, token, organisation_id, study["id"])
    created = await _create_session(
        client, token, organisation_id, dataset["id"], b"this is not npy"
    )
    content = await client.put(
        f"{SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset['id'])}"
        f"/{created['upload_session_id']}/content",
        headers=bearer(token),
        content=b"this is not npy",
    )
    assert content.status_code == 204
    finalise = await client.post(
        FINALISE.format(
            organisation_id=organisation_id,
            dataset_id=dataset["id"],
            session_id=created["upload_session_id"],
        ),
        headers=bearer(token),
    )
    assert finalise.status_code == 422
    assert finalise.json()["error"]["code"] == "invalid_recording"

    after = await client.get(
        f"/api/v1/organisations/{organisation_id}/datasets/{dataset['id']}",
        headers=bearer(token),
    )
    assert after.json()["status"] == "rejected"


async def test_checksum_mismatch_is_rejected(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(client, token, organisation_id, study["id"])
    data = _npy_bytes()
    wrong_checksum = hashlib.sha256(b"something else").hexdigest()
    created = await _create_session(
        client,
        token,
        organisation_id,
        dataset["id"],
        data,
        expected_checksum_sha256=wrong_checksum,
    )
    content = await client.put(
        f"{SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset['id'])}"
        f"/{created['upload_session_id']}/content",
        headers=bearer(token),
        content=data,
    )
    assert content.status_code == 204
    finalise = await client.post(
        FINALISE.format(
            organisation_id=organisation_id,
            dataset_id=dataset["id"],
            session_id=created["upload_session_id"],
        ),
        headers=bearer(token),
    )
    assert finalise.status_code == 422
    assert finalise.json()["error"]["code"] == "invalid_recording"
    after = await client.get(
        f"/api/v1/organisations/{organisation_id}/datasets/{dataset['id']}",
        headers=bearer(token),
    )
    assert after.json()["status"] == "rejected"


async def test_declared_size_mismatch_is_rejected(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(client, token, organisation_id, study["id"])
    data = _npy_bytes()
    created = await _create_session(
        client, token, organisation_id, dataset["id"], data, file_size_bytes=len(data) + 999
    )
    content = await client.put(
        f"{SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset['id'])}"
        f"/{created['upload_session_id']}/content",
        headers=bearer(token),
        content=data,
    )
    assert content.status_code == 204
    finalise = await client.post(
        FINALISE.format(
            organisation_id=organisation_id,
            dataset_id=dataset["id"],
            session_id=created["upload_session_id"],
        ),
        headers=bearer(token),
    )
    assert finalise.status_code == 422
    assert "size" in finalise.json()["error"]["message"]


async def test_finalise_without_content_returns_409(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(client, token, organisation_id, study["id"])
    created = await _create_session(client, token, organisation_id, dataset["id"], b"x")
    finalise = await client.post(
        FINALISE.format(
            organisation_id=organisation_id,
            dataset_id=dataset["id"],
            session_id=created["upload_session_id"],
        ),
        headers=bearer(token),
    )
    assert finalise.status_code == 409
    assert finalise.json()["error"]["code"] == "upload_content_missing"


async def test_expired_session_cannot_be_finalised(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(client, token, organisation_id, study["id"])
    created = await _create_session(client, token, organisation_id, dataset["id"], b"x")
    db.execute(
        "UPDATE upload_sessions SET expires_at = now() - interval '1 hour' WHERE id = %s",
        (created["upload_session_id"],),
    )
    finalise = await client.post(
        FINALISE.format(
            organisation_id=organisation_id,
            dataset_id=dataset["id"],
            session_id=created["upload_session_id"],
        ),
        headers=bearer(token),
    )
    assert finalise.status_code == 409
    assert finalise.json()["error"]["code"] == "upload_session_expired"


async def test_ready_dataset_cannot_accept_another_upload(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(client, token, organisation_id, study["id"])
    data = _npy_bytes()
    await _upload_and_finalise(client, token, organisation_id, dataset["id"], data)
    response = await client.post(
        SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=bearer(token),
        json={
            "file_name": "another.npy",
            "file_size_bytes": len(data),
            "manifest": _manifest(),
        },
    )
    assert response.status_code == 409
    assert response.json()["error"]["code"] == "dataset_not_uploadable"


async def test_unknown_format_is_rejected(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(client, token, organisation_id, study["id"])
    response = await client.post(
        SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=bearer(token),
        json={
            "file_name": "rec.edf",
            "file_size_bytes": 10,
            "format": "edf",
            "manifest": _manifest(),
        },
    )
    assert response.status_code == 422


async def test_pickled_object_array_is_rejected_by_the_api(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(client, token, organisation_id, study["id"])
    buffer = io.BytesIO()
    np.save(
        buffer,
        np.array([object()] * 8, dtype=object).reshape(2, 4),
        allow_pickle=True,
    )
    pickled = buffer.getvalue()
    created = await _create_session(client, token, organisation_id, dataset["id"], pickled)
    content = await client.put(
        f"{SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset['id'])}"
        f"/{created['upload_session_id']}/content",
        headers=bearer(token),
        content=pickled,
    )
    assert content.status_code == 204
    finalise = await client.post(
        FINALISE.format(
            organisation_id=organisation_id,
            dataset_id=dataset["id"],
            session_id=created["upload_session_id"],
        ),
        headers=bearer(token),
    )
    assert finalise.status_code == 422
    assert finalise.json()["error"]["code"] == "invalid_recording"


async def test_content_endpoint_accepts_bodies_larger_than_json_limit(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(client, token, organisation_id, study["id"])
    data = _npy_bytes(channels=256, samples=1024, dtype="float64")
    assert len(data) > 1024 * 1024
    created = await _create_session(client, token, organisation_id, dataset["id"], data)
    content = await client.put(
        f"{SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset['id'])}"
        f"/{created['upload_session_id']}/content",
        headers=bearer(token),
        content=data,
    )
    assert content.status_code == 204, content.text

    oversized = await client.post(
        SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=bearer(token),
        json={
            "file_name": "rec.npy",
            "file_size_bytes": 10,
            "manifest": _manifest(),
        },
    )
    assert oversized.status_code == 201


async def test_service_size_limit_rejects_oversized_upload(
    db: psycopg.Connection[dict[str, Any]],
) -> None:
    from httpx import ASGITransport

    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    app = create_app(make_settings(max_upload_bytes=64))
    async with app.router.lifespan_context(app):
        transport = ASGITransport(app=app)
        async with AsyncClient(transport=transport, base_url="http://testserver") as small_client:
            token = await issue_token(small_client, subject="researcher")
            study = await _create_study(small_client, token, organisation_id, name="Tiny")
            dataset = await _register_dataset(small_client, token, organisation_id, study["id"])
            data = _npy_bytes()
            created = await _create_session(
                small_client, token, organisation_id, dataset["id"], data
            )
            content = await small_client.put(
                f"{SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset['id'])}"
                f"/{created['upload_session_id']}/content",
                headers=bearer(token),
                content=data,
            )
            assert content.status_code == 413
            assert content.json()["error"]["code"] == "payload_too_large"


async def test_missing_session_returns_404(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(client, token, organisation_id, study["id"])
    finalise = await client.post(
        FINALISE.format(
            organisation_id=organisation_id,
            dataset_id=dataset["id"],
            session_id=MISSING,
        ),
        headers=bearer(token),
    )
    assert finalise.status_code == 404
