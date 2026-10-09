"""API tests for the signal display endpoint."""

from __future__ import annotations

import io
from typing import Any, cast
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
SIGNALS = "/api/v1/organisations/{organisation_id}/datasets/{dataset_id}/signals"
MISSING = "00000000-0000-4000-8000-000000000000"

FREQUENCY = 256.0
SAMPLES = int(FREQUENCY)


def _seed_org(db: psycopg.Connection[dict[str, Any]], subject: str = "researcher") -> UUID:
    user_id = create_user(db, subject=subject)
    organisation_id = create_organisation(db, name="Lab")
    create_membership(db, organisation_id=organisation_id, user_id=user_id, role="researcher")
    return organisation_id


def _npy_bytes(values: np.ndarray) -> bytes:
    buffer = io.BytesIO()
    np.save(buffer, values, allow_pickle=False)
    return buffer.getvalue()


async def _ready_dataset(
    client: AsyncClient,
    token: str,
    organisation_id: UUID,
    values: np.ndarray,
    *,
    channel_units: list[str] | None = None,
) -> dict[str, Any]:
    channel_count = int(values.shape[0])
    names = [f"ch{index}" for index in range(channel_count)]
    units = channel_units or ["uV"] * channel_count

    study = await client.post(
        STUDIES.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={"name": "Study"},
    )
    assert study.status_code == 201, study.text
    dataset = await client.post(
        DATASETS.format(organisation_id=organisation_id, study_id=study.json()["id"]),
        headers=bearer(token),
        json={"name": "Recording"},
    )
    assert dataset.status_code == 201, dataset.text
    dataset_id = dataset.json()["id"]

    data = _npy_bytes(values)
    manifest = {
        "sampling_frequency_hz": FREQUENCY,
        "channel_names": names,
        "channel_units": units,
    }
    session = await client.post(
        SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset_id),
        headers=bearer(token),
        json={"file_name": "recording.npy", "file_size_bytes": len(data), "manifest": manifest},
    )
    assert session.status_code == 201, session.text
    session_id = session.json()["upload_session_id"]
    content = await client.put(
        f"{SESSIONS.format(organisation_id=organisation_id, dataset_id=dataset_id)}"
        f"/{session_id}/content",
        headers=bearer(token),
        content=data,
    )
    assert content.status_code == 204, content.text
    finalised = await client.post(
        FINALISE.format(
            organisation_id=organisation_id, dataset_id=dataset_id, session_id=session_id
        ),
        headers=bearer(token),
    )
    assert finalised.status_code == 200, finalised.text
    return cast(dict[str, Any], finalised.json()["dataset"])


def _values() -> np.ndarray:
    return np.vstack(
        [np.arange(FREQUENCY, dtype=np.float64), np.arange(FREQUENCY, dtype=np.float64) * 2]
    )


async def test_returns_the_default_channel_window(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    organisation_id = _seed_org(db)
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(client, token, organisation_id, np.zeros((2, 256)))

    response = await client.get(
        SIGNALS.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=bearer(token),
        params={"start_seconds": 0.0, "end_seconds": 1.0},
    )

    assert response.status_code == 200, response.text
    body = response.json()
    assert body["dataset_id"] == dataset["id"]
    assert body["channel_indices"] == [0]
    assert body["points_per_channel"] == 256
    assert body["downsampled"] is False
    assert body["time_seconds"][1] == 1.0 / FREQUENCY
    assert len(body["channels"][0]["samples"]) == 256
    assert body["channels"][0]["channel_name"] == "ch0"


async def test_returns_the_requested_channel_group(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    organisation_id = _seed_org(db)
    token = await issue_token(client, subject="researcher")
    values = np.vstack(
        [np.arange(FREQUENCY, dtype=np.float64), np.arange(FREQUENCY, dtype=np.float64) * 2]
    )
    dataset = await _ready_dataset(client, token, organisation_id, values)

    response = await client.get(
        SIGNALS.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=bearer(token),
        params={"start_seconds": 0.0, "end_seconds": 1.0, "channels": "0,1"},
    )

    assert response.status_code == 200, response.text
    body = response.json()
    assert body["channel_indices"] == [0, 1]
    assert body["channels"][1]["samples"][4] == 8.0


async def test_foreign_dataset_is_not_found(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    organisation_id = _seed_org(db, "researcher")
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(
        client, token, organisation_id, np.zeros((1, SAMPLES), dtype=np.float64)
    )

    other_user = create_user(db, subject="intruder")
    other_org = create_organisation(db, name="Other")
    create_membership(db, organisation_id=other_org, user_id=other_user, role="owner")
    other_token = await issue_token(client, subject="intruder")

    response = await client.get(
        SIGNALS.format(organisation_id=other_org, dataset_id=dataset["id"]),
        headers=bearer(other_token),
        params={"end_seconds": 1.0},
    )

    assert response.status_code == 404


async def test_non_ready_dataset_conflicts(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    organisation_id = _seed_org(db)
    token = await issue_token(client, subject="researcher")
    study = await client.post(
        STUDIES.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={"name": "Study"},
    )
    dataset = await client.post(
        DATASETS.format(organisation_id=organisation_id, study_id=study.json()["id"]),
        headers=bearer(token),
        json={"name": "Pending"},
    )

    response = await client.get(
        SIGNALS.format(organisation_id=organisation_id, dataset_id=dataset.json()["id"]),
        headers=bearer(token),
        params={"end_seconds": 1.0},
    )

    assert response.status_code == 409
    assert response.json()["error"]["code"] == "dataset_not_ready"


async def test_missing_dataset_is_not_found(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    organisation_id = _seed_org(db)
    token = await issue_token(client, subject="researcher")

    response = await client.get(
        SIGNALS.format(organisation_id=organisation_id, dataset_id=MISSING),
        headers=bearer(token),
        params={"end_seconds": 1.0},
    )

    assert response.status_code == 404


async def test_invalid_window_is_rejected(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    organisation_id = _seed_org(db)
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(
        client, token, organisation_id, np.zeros((1, SAMPLES), dtype=np.float64)
    )

    response = await client.get(
        SIGNALS.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=bearer(token),
        params={"start_seconds": 0.0, "end_seconds": 5.0},
    )

    assert response.status_code == 422
    assert response.json()["error"]["code"] == "invalid_signal_window"


async def test_invalid_channel_selection_is_rejected(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    organisation_id = _seed_org(db)
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(
        client, token, organisation_id, np.zeros((1, SAMPLES), dtype=np.float64)
    )

    response = await client.get(
        SIGNALS.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=bearer(token),
        params={"end_seconds": 0.5, "channels": "9"},
    )

    assert response.status_code == 422
    assert response.json()["error"]["code"] == "invalid_signal_window"


async def test_non_integer_channel_is_rejected(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    organisation_id = _seed_org(db)
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(
        client, token, organisation_id, np.zeros((1, SAMPLES), dtype=np.float64)
    )

    response = await client.get(
        SIGNALS.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=bearer(token),
        params={"end_seconds": 0.5, "channels": "abc"},
    )

    assert response.status_code == 422
    assert response.json()["error"]["code"] == "invalid_signal_window"


async def test_end_seconds_is_required(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    organisation_id = _seed_org(db)
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(
        client, token, organisation_id, np.zeros((1, SAMPLES), dtype=np.float64)
    )

    response = await client.get(
        SIGNALS.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=bearer(token),
    )

    assert response.status_code == 422
