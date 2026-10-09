"""API tests for the organisation dashboard summary."""

from __future__ import annotations

import io
from typing import Any
from uuid import UUID

import numpy as np
import psycopg
from httpx import AsyncClient

from tests.conftest import (
    bearer,
    create_membership,
    create_organisation,
    create_user,
    issue_token,
)

STUDIES = "/api/v1/organisations/{organisation_id}/studies"
DATASETS = "/api/v1/organisations/{organisation_id}/studies/{study_id}/datasets"
SESSIONS = "/api/v1/organisations/{organisation_id}/datasets/{dataset_id}/upload-sessions"
FINALISE = (
    "/api/v1/organisations/{organisation_id}/datasets/{dataset_id}"
    "/upload-sessions/{session_id}/finalise"
)
ANALYSES = "/api/v1/organisations/{organisation_id}/datasets/{dataset_id}/analyses"
SUMMARY = "/api/v1/organisations/{organisation_id}/summary"


def _seed_org(db: psycopg.Connection[dict[str, Any]], subject: str = "researcher") -> UUID:
    user_id = create_user(db, subject=subject)
    organisation_id = create_organisation(db, name="Lab")
    create_membership(db, organisation_id=organisation_id, user_id=user_id, role="researcher")
    return organisation_id


async def _create_study(client: AsyncClient, token: str, organisation_id: UUID) -> str:
    response = await client.post(
        STUDIES.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={"name": "Study"},
    )
    assert response.status_code == 201, response.text
    return str(response.json()["id"])


async def _ready_dataset(
    client: AsyncClient, token: str, organisation_id: UUID, study_id: str
) -> str:
    dataset = await client.post(
        DATASETS.format(organisation_id=organisation_id, study_id=study_id),
        headers=bearer(token),
        json={"name": "Recording"},
    )
    assert dataset.status_code == 201, dataset.text
    dataset_id = dataset.json()["id"]

    buffer = io.BytesIO()
    np.save(buffer, np.zeros((1, 32), dtype=np.float32), allow_pickle=False)
    data = buffer.getvalue()
    session = await client.post(
        SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset_id),
        headers=bearer(token),
        json={
            "file_name": "recording.npy",
            "file_size_bytes": len(data),
            "manifest": {
                "sampling_frequency_hz": 256.0,
                "channel_names": ["F3"],
                "channel_units": ["uV"],
            },
        },
    )
    assert session.status_code == 201, session.text
    session_id = session.json()["upload_session_id"]
    await client.put(
        f"{SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset_id)}"
        f"/{session_id}/content",
        headers=bearer(token),
        content=data,
    )
    finalised = await client.post(
        FINALISE.format(
            organisation_id=organisation_id, dataset_id=dataset_id, session_id=session_id
        ),
        headers=bearer(token),
    )
    assert finalised.status_code == 200, finalised.text
    return str(dataset_id)


async def test_summary_counts_resources_and_statuses(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    organisation_id = _seed_org(db)
    token = await issue_token(client, subject="researcher")

    first = await client.post(
        STUDIES.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={"name": "First"},
    )
    second = await client.post(
        STUDIES.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={"name": "Second"},
    )
    study_id = first.json()["id"]
    await client.post(
        DATASETS.format(organisation_id=organisation_id, study_id=study_id),
        headers=bearer(token),
        json={"name": "Pending"},
    )

    pending = await client.get(
        SUMMARY.format(organisation_id=organisation_id), headers=bearer(token)
    )
    assert pending.status_code == 200, pending.text
    body = pending.json()
    assert body["study_count"] == 2
    assert body["active_study_count"] == 2
    assert body["dataset_count"] == 1
    assert body["ready_dataset_count"] == 0
    assert body["analysis_count"] == 0
    assert body["analyses_by_status"] == {
        "queued": 0,
        "running": 0,
        "succeeded": 0,
        "failed": 0,
        "cancelled": 0,
    }

    await client.post(
        f"{STUDIES.format(organisation_id=organisation_id)}/{second.json()['id']}/archive",
        headers=bearer(token),
    )
    assert study_id
    dataset_id = await _ready_dataset(client, token, organisation_id, study_id)
    submitted = await client.post(
        ANALYSES.format(organisation_id=organisation_id, dataset_id=dataset_id),
        headers={**bearer(token), "Idempotency-Key": "summary-key"},
        json={"analysis_type": "descriptive_statistics"},
    )
    assert submitted.status_code == 202, submitted.text

    updated = await client.get(
        SUMMARY.format(organisation_id=organisation_id), headers=bearer(token)
    )
    assert updated.status_code == 200
    body = updated.json()
    assert body["active_study_count"] == 1
    assert body["ready_dataset_count"] == 1
    assert body["analysis_count"] == 1
    assert body["analyses_by_status"]["queued"] == 1


async def test_summary_is_scoped_to_the_organisation(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    owner_id = create_user(db, subject="owner")
    org_a = create_organisation(db, name="A")
    create_membership(db, organisation_id=org_a, user_id=owner_id, role="owner")
    token_a = await issue_token(client, subject="owner")
    await client.post(
        STUDIES.format(organisation_id=org_a), headers=bearer(token_a), json={"name": "Study"}
    )

    intruder = create_user(db, subject="intruder")
    org_b = create_organisation(db, name="B")
    create_membership(db, organisation_id=org_b, user_id=intruder, role="owner")
    token_b = await issue_token(client, subject="intruder")

    response = await client.get(SUMMARY.format(organisation_id=org_a), headers=bearer(token_b))

    assert response.status_code == 404
