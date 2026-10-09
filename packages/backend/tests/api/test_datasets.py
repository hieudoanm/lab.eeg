"""API tests for the dataset endpoints."""

from __future__ import annotations

from typing import Any, cast
from uuid import UUID

import psycopg
from httpx import AsyncClient

from tests.conftest import (
    bearer,
    create_membership,
    create_organisation,
    create_user,
    issue_token,
)

DATASETS = "/api/v1/organisations/{organisation_id}/studies/{study_id}/datasets"
MISSING = "00000000-0000-4000-8000-000000000000"


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
        f"/api/v1/organisations/{organisation_id}/studies",
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
    **extra: object,
) -> dict[str, Any]:
    response = await client.post(
        DATASETS.format(organisation_id=organisation_id, study_id=study_id),
        headers=bearer(token),
        json={"name": name, **extra},
    )
    assert response.status_code == 201, response.text
    return cast(dict[str, Any], response.json())


async def test_register_dataset_succeeds(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    body = await _register_dataset(
        client,
        token,
        organisation_id,
        study["id"],
        metadata={"signal_type": "eeg"},
        source_description="  Generated synthetic data  ",
    )
    assert body["study_id"] == study["id"]
    assert body["status"] == "pending_upload"
    assert body["format"] == "npy"
    assert body["description"] is None
    assert body["source_description"] == "Generated synthetic data"
    assert body["sampling_frequency_hz"] is None
    assert "storage_key" not in body
    assert "metadata" not in body
    assert "organisation_id" not in body


async def test_register_dataset_requires_authentication(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    client.cookies.clear()
    response = await client.post(
        DATASETS.format(organisation_id=organisation_id, study_id=study["id"]),
        json={"name": "Anonymous"},
    )
    assert response.status_code == 401


async def test_viewer_cannot_register_dataset(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="owner", role="owner")
    token = await issue_token(client, subject="owner")
    study = await _create_study(client, token, organisation_id)
    viewer_id = create_user(db, subject="viewer-data")
    create_membership(db, organisation_id=organisation_id, user_id=viewer_id, role="viewer")
    viewer_token = await issue_token(client, subject="viewer-data")
    response = await client.post(
        DATASETS.format(organisation_id=organisation_id, study_id=study["id"]),
        headers=bearer(viewer_token),
        json={"name": "Denied"},
    )
    assert response.status_code == 403
    assert response.json()["error"]["code"] == "insufficient_role"


async def test_register_dataset_requires_active_study(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    await client.post(
        f"/api/v1/organisations/{organisation_id}/studies/{study['id']}/archive",
        headers=bearer(token),
    )
    response = await client.post(
        DATASETS.format(organisation_id=organisation_id, study_id=study["id"]),
        headers=bearer(token),
        json={"name": "Too late"},
    )
    assert response.status_code == 409
    assert response.json()["error"]["code"] == "archived_study"


async def test_register_dataset_in_missing_study_returns_404(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    response = await client.post(
        DATASETS.format(organisation_id=organisation_id, study_id=MISSING),
        headers=bearer(token),
        json={"name": "Nowhere"},
    )
    assert response.status_code == 404


async def test_register_dataset_rejects_unknown_format(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    response = await client.post(
        DATASETS.format(organisation_id=organisation_id, study_id=study["id"]),
        headers=bearer(token),
        json={"name": "Bad", "format": "edf"},
    )
    assert response.status_code == 422


async def test_list_datasets_is_empty_then_lists(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    url = DATASETS.format(organisation_id=organisation_id, study_id=study["id"])
    empty = await client.get(url, headers=bearer(token))
    assert empty.status_code == 200
    assert empty.json() == {"items": [], "next_cursor": None, "has_more": False}
    await _register_dataset(client, token, organisation_id, study["id"], name="Session 1")
    listing = await client.get(url, headers=bearer(token))
    assert listing.status_code == 200
    assert [item["name"] for item in listing.json()["items"]] == ["Session 1"]


async def test_list_datasets_in_missing_study_returns_404(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    response = await client.get(
        DATASETS.format(organisation_id=organisation_id, study_id=MISSING),
        headers=bearer(token),
    )
    assert response.status_code == 404


async def test_read_dataset_metadata(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(
        client,
        token,
        organisation_id,
        study["id"],
        name="Participant 1",
        description="Synthetic EEG",
    )
    response = await client.get(
        f"/api/v1/organisations/{organisation_id}/datasets/{dataset['id']}",
        headers=bearer(token),
    )
    assert response.status_code == 200
    body = response.json()
    assert body["name"] == "Participant 1"
    assert body["description"] == "Synthetic EEG"
    assert body["status"] == "pending_upload"
    assert body["created_at"].endswith("Z")


async def test_viewer_can_read_dataset_metadata(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="owner", role="owner")
    token = await issue_token(client, subject="owner")
    study = await _create_study(client, token, organisation_id)
    dataset = await _register_dataset(client, token, organisation_id, study["id"])
    viewer_id = create_user(db, subject="viewer-read")
    create_membership(db, organisation_id=organisation_id, user_id=viewer_id, role="viewer")
    viewer_token = await issue_token(client, subject="viewer-read")
    response = await client.get(
        f"/api/v1/organisations/{organisation_id}/datasets/{dataset['id']}",
        headers=bearer(viewer_token),
    )
    assert response.status_code == 200
    assert response.json()["id"] == dataset["id"]


async def test_dataset_is_scoped_to_organisation(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, org_a = _seed_member(
        db, subject="researcher-a", role="researcher", organisation_name="Organisation A"
    )
    token_a = await issue_token(client, subject="researcher-a")
    study = await _create_study(client, token_a, org_a)
    dataset = await _register_dataset(client, token_a, org_a, study["id"])
    _, org_b = _seed_member(
        db, subject="researcher-b", role="researcher", organisation_name="Organisation B"
    )
    token_b = await issue_token(client, subject="researcher-b")
    response = await client.get(
        f"/api/v1/organisations/{org_b}/datasets/{dataset['id']}", headers=bearer(token_b)
    )
    assert response.status_code == 404
