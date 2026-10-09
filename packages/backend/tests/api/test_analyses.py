"""API tests for analysis submission, status, results, and cancellation."""

from __future__ import annotations

import io
from typing import Any, cast
from uuid import UUID

import numpy as np
import psycopg
from httpx import AsyncClient
from psycopg.types.json import Jsonb

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
ORG_ANALYSES = "/api/v1/organisations/{organisation_id}/analyses"
ANALYSIS = "/api/v1/organisations/{organisation_id}/analyses/{analysis_id}"
RESULTS = "/api/v1/organisations/{organisation_id}/analyses/{analysis_id}/results"
CANCEL = "/api/v1/organisations/{organisation_id}/analyses/{analysis_id}/cancel"
MISSING = "00000000-0000-4000-8000-000000000000"

SUBMIT_BODY: dict[str, Any] = {
    "analysis_type": "descriptive_statistics",
    "algorithm_version": "1.0.0",
    "parameters": {
        "include_mean": True,
        "include_standard_deviation": True,
        "include_min_max": True,
        "include_peak_to_peak": True,
    },
}


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


def _npy_bytes(*, channels: int = 2, samples: int = 8) -> bytes:
    buffer = io.BytesIO()
    np.save(buffer, np.zeros((channels, samples), dtype="float32"), allow_pickle=False)
    return buffer.getvalue()


async def _ready_dataset(client: AsyncClient, token: str, organisation_id: UUID) -> dict[str, Any]:
    study = await client.post(
        STUDIES.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={"name": "Study"},
    )
    assert study.status_code == 201, study.text
    study_id = study.json()["id"]
    dataset = await client.post(
        DATASETS.format(organisation_id=organisation_id, study_id=study_id),
        headers=bearer(token),
        json={"name": "Recording"},
    )
    assert dataset.status_code == 201, dataset.text
    dataset_id = dataset.json()["id"]

    data = _npy_bytes()
    manifest = {
        "sampling_frequency_hz": 256.0,
        "channel_names": ["F3", "F4"],
        "channel_units": ["uV", "uV"],
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
    ready = finalised.json()["dataset"]
    assert ready["status"] == "ready"
    return cast(dict[str, Any], ready)


def _headers(token: str, key: str = "idem-key-1") -> dict[str, str]:
    return {**bearer(token), "Idempotency-Key": key}


async def _submit(
    client: AsyncClient,
    token: str,
    organisation_id: UUID,
    dataset_id: str,
    *,
    key: str = "idem-key-1",
    body: dict[str, Any] | None = None,
) -> dict[str, Any]:
    response = await client.post(
        ANALYSES.format(organisation_id=organisation_id, dataset_id=dataset_id),
        headers=_headers(token, key),
        json=body if body is not None else SUBMIT_BODY,
    )
    assert response.status_code == 202, response.text
    return cast(dict[str, Any], response.json())


async def test_submit_creates_job_and_outbox_event_transactionally(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(client, token, organisation_id)

    job = await _submit(client, token, organisation_id, dataset["id"])

    assert job["status"] == "queued"
    assert job["analysis_type"] == "descriptive_statistics"
    assert job["algorithm_version"] == "1.0.0"
    assert job["attempt_count"] == 0
    assert job["max_attempts"] == 3
    assert job["requested_at"].endswith("Z")
    assert job["error"] is None

    jobs = db.execute("SELECT COUNT(*) AS count FROM analysis_jobs").fetchone()
    assert jobs is not None and jobs["count"] == 1
    outbox = db.execute(
        "SELECT COUNT(*) AS count FROM outbox_events WHERE published_at IS NULL"
    ).fetchone()
    assert outbox is not None and outbox["count"] == 1
    record = db.execute("SELECT resource_id, response_status FROM idempotency_keys").fetchone()
    assert record is not None
    assert record["resource_id"] == UUID(job["id"])
    assert record["response_status"] == 202


async def test_identical_idempotent_request_replays_the_job(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(client, token, organisation_id)

    first = await _submit(client, token, organisation_id, dataset["id"], key="same")
    second = await _submit(client, token, organisation_id, dataset["id"], key="same")

    assert second["id"] == first["id"]
    jobs = db.execute("SELECT COUNT(*) AS count FROM analysis_jobs").fetchone()
    assert jobs is not None and jobs["count"] == 1
    outbox = db.execute("SELECT COUNT(*) AS count FROM outbox_events").fetchone()
    assert outbox is not None and outbox["count"] == 1


async def test_reused_key_with_different_request_is_rejected(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(client, token, organisation_id)

    await _submit(client, token, organisation_id, dataset["id"], key="clash")
    different = {**SUBMIT_BODY, "parameters": {"include_mean": False}}
    response = await client.post(
        ANALYSES.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=_headers(token, "clash"),
        json=different,
    )

    assert response.status_code == 409
    assert response.json()["error"]["code"] == "idempotency_key_reuse"
    jobs = db.execute("SELECT COUNT(*) AS count FROM analysis_jobs").fetchone()
    assert jobs is not None and jobs["count"] == 1


async def test_submit_requires_an_idempotency_key(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(client, token, organisation_id)

    response = await client.post(
        ANALYSES.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=bearer(token),
        json=SUBMIT_BODY,
    )

    assert response.status_code == 422


async def test_non_ready_dataset_cannot_be_analysed(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
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

    response = await client.post(
        ANALYSES.format(organisation_id=organisation_id, dataset_id=dataset.json()["id"]),
        headers=_headers(token),
        json=SUBMIT_BODY,
    )

    assert response.status_code == 409
    assert response.json()["error"]["code"] == "dataset_not_ready"


async def test_unsupported_algorithm_version_is_rejected(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(client, token, organisation_id)

    response = await client.post(
        ANALYSES.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=_headers(token),
        json={**SUBMIT_BODY, "algorithm_version": "9.9.9"},
    )

    assert response.status_code == 422


async def test_unknown_parameter_field_is_rejected(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(client, token, organisation_id)

    response = await client.post(
        ANALYSES.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=_headers(token),
        json={**SUBMIT_BODY, "parameters": {"bogus": True}},
    )

    assert response.status_code == 422


async def test_viewer_cannot_submit_an_analysis(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="owner", role="owner")
    token = await issue_token(client, subject="owner")
    dataset = await _ready_dataset(client, token, organisation_id)
    viewer_id = create_user(db, subject="viewer-analysis")
    create_membership(db, organisation_id=organisation_id, user_id=viewer_id, role="viewer")
    viewer_token = await issue_token(client, subject="viewer-analysis")

    response = await client.post(
        ANALYSES.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=_headers(viewer_token),
        json=SUBMIT_BODY,
    )

    assert response.status_code == 403
    assert response.json()["error"]["code"] == "insufficient_role"


async def test_results_are_not_ready_while_queued(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(client, token, organisation_id)
    job = await _submit(client, token, organisation_id, dataset["id"])

    response = await client.get(
        RESULTS.format(organisation_id=organisation_id, analysis_id=job["id"]),
        headers=bearer(token),
    )

    assert response.status_code == 409
    assert response.json()["error"]["code"] == "results_not_ready"


async def test_analyses_are_scoped_to_the_organisation(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, org_a = _seed_member(
        db, subject="researcher-a", role="researcher", organisation_name="Org A"
    )
    token_a = await issue_token(client, subject="researcher-a")
    dataset = await _ready_dataset(client, token_a, org_a)
    job = await _submit(client, token_a, org_a, dataset["id"])

    _, org_b = _seed_member(
        db, subject="researcher-b", role="researcher", organisation_name="Org B"
    )
    token_b = await issue_token(client, subject="researcher-b")

    read = await client.get(
        ANALYSIS.format(organisation_id=org_b, analysis_id=job["id"]),
        headers=bearer(token_b),
    )
    assert read.status_code == 404

    other_org_dataset = await client.post(
        ANALYSES.format(organisation_id=org_b, dataset_id=dataset["id"]),
        headers=_headers(token_b),
        json=SUBMIT_BODY,
    )
    assert other_org_dataset.status_code == 404


async def test_cancel_queued_job_is_idempotent_and_reflects_state(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(client, token, organisation_id)
    job = await _submit(client, token, organisation_id, dataset["id"])

    first = await client.post(
        CANCEL.format(organisation_id=organisation_id, analysis_id=job["id"]),
        headers=bearer(token),
    )
    assert first.status_code == 200, first.text
    assert first.json()["status"] == "cancelled"

    second = await client.post(
        CANCEL.format(organisation_id=organisation_id, analysis_id=job["id"]),
        headers=bearer(token),
    )
    assert second.status_code == 200
    assert second.json()["status"] == "cancelled"

    results = await client.get(
        RESULTS.format(organisation_id=organisation_id, analysis_id=job["id"]),
        headers=bearer(token),
    )
    assert results.status_code == 409
    assert results.json()["error"]["code"] == "result_not_available"


async def test_status_exposes_a_safe_error_for_failed_jobs(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(client, token, organisation_id)
    job = await _submit(client, token, organisation_id, dataset["id"])

    db.execute(
        """
        UPDATE analysis_jobs
        SET status = 'failed', completed_at = now(),
            error_code = 'analysis_input_invalid',
            error_message = 'The recording could not be analysed.'
        WHERE id = %s
        """,
        (job["id"],),
    )
    response = await client.get(
        ANALYSIS.format(organisation_id=organisation_id, analysis_id=job["id"]),
        headers=bearer(token),
    )

    assert response.status_code == 200
    body = response.json()
    assert body["status"] == "failed"
    assert body["error"]["code"] == "analysis_input_invalid"
    assert "storage" not in body["error"]["message"].lower()


async def test_results_and_cancel_behaviour_for_a_succeeded_job(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(client, token, organisation_id)
    job = await _submit(client, token, organisation_id, dataset["id"])

    results_payload = {
        "channels": [
            {
                "channel_index": 0,
                "channel_name": "F3",
                "unit": "uV",
                "mean": 0.0,
                "standard_deviation": 1.0,
                "minimum": -1.0,
                "maximum": 1.0,
                "peak_to_peak": 2.0,
                "valid_sample_count": 8,
            }
        ]
    }
    provenance = {
        "input_checksum_sha256": dataset["checksum_sha256"],
        "sampling_frequency_hz": 256.0,
        "channel_count": 2,
        "sample_count": 8,
        "parameters": SUBMIT_BODY["parameters"],
    }
    db.execute(
        """
        UPDATE analysis_jobs
        SET status = 'succeeded', attempt_count = 1, completed_at = now()
        WHERE id = %s
        """,
        (job["id"],),
    )
    db.execute(
        """
        INSERT INTO analysis_results (
            organisation_id, job_id, dataset_id, dataset_checksum_sha256,
            algorithm_version, result_schema_version, results, provenance
        )
        VALUES (%s, %s, %s, %s, '1.0.0', '1.0.0', %s, %s)
        """,
        (
            organisation_id,
            job["id"],
            dataset["id"],
            dataset["checksum_sha256"],
            Jsonb(results_payload),
            Jsonb(provenance),
        ),
    )

    results = await client.get(
        RESULTS.format(organisation_id=organisation_id, analysis_id=job["id"]),
        headers=bearer(token),
    )
    assert results.status_code == 200, results.text
    body = results.json()
    assert body["analysis_id"] == job["id"]
    assert body["result_schema_version"] == "1.0.0"
    assert body["channels"][0]["channel_name"] == "F3"
    assert body["provenance"]["input_checksum_sha256"] == dataset["checksum_sha256"]

    cancel = await client.post(
        CANCEL.format(organisation_id=organisation_id, analysis_id=job["id"]),
        headers=bearer(token),
    )
    assert cancel.status_code == 409
    assert cancel.json()["error"]["code"] == "analysis_not_cancellable"


async def test_list_analyses_returns_the_job(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(client, token, organisation_id)
    job = await _submit(client, token, organisation_id, dataset["id"])

    response = await client.get(
        ANALYSES.format(organisation_id=organisation_id, dataset_id=dataset["id"]),
        headers=bearer(token),
    )

    assert response.status_code == 200, response.text
    body = response.json()
    assert [item["id"] for item in body["items"]] == [job["id"]]
    assert body["has_more"] is False


async def test_missing_analysis_returns_404(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")

    response = await client.get(
        ANALYSIS.format(organisation_id=organisation_id, analysis_id=MISSING),
        headers=bearer(token),
    )

    assert response.status_code == 404


async def test_org_analysis_list_spans_datasets_and_filters(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    first = await _ready_dataset(client, token, organisation_id)
    second = await _ready_dataset(client, token, organisation_id)
    first_job = await _submit(client, token, organisation_id, first["id"], key="first")
    second_job = await _submit(client, token, organisation_id, second["id"], key="second")

    listed = await client.get(
        ORG_ANALYSES.format(organisation_id=organisation_id), headers=bearer(token)
    )
    assert listed.status_code == 200, listed.text
    assert {item["id"] for item in listed.json()["items"]} == {first_job["id"], second_job["id"]}

    by_dataset = await client.get(
        ORG_ANALYSES.format(organisation_id=organisation_id),
        headers=bearer(token),
        params={"dataset_id": first["id"]},
    )
    assert [item["id"] for item in by_dataset.json()["items"]] == [first_job["id"]]

    by_study = await client.get(
        ORG_ANALYSES.format(organisation_id=organisation_id),
        headers=bearer(token),
        params={"study_id": second["study_id"]},
    )
    assert [item["id"] for item in by_study.json()["items"]] == [second_job["id"]]

    queued = await client.get(
        ORG_ANALYSES.format(organisation_id=organisation_id),
        headers=bearer(token),
        params={"status": "queued"},
    )
    assert len(queued.json()["items"]) == 2

    cancelled = await client.get(
        ORG_ANALYSES.format(organisation_id=organisation_id),
        headers=bearer(token),
        params={"status": "cancelled"},
    )
    assert cancelled.json()["items"] == []


async def test_org_analysis_list_paginates(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    dataset = await _ready_dataset(client, token, organisation_id)
    await _submit(client, token, organisation_id, dataset["id"], key="one")
    await _submit(client, token, organisation_id, dataset["id"], key="two")

    first_page = await client.get(
        ORG_ANALYSES.format(organisation_id=organisation_id),
        headers=bearer(token),
        params={"limit": 1},
    )
    body = first_page.json()
    assert len(body["items"]) == 1
    assert body["has_more"] is True
    assert body["next_cursor"]

    second_page = await client.get(
        ORG_ANALYSES.format(organisation_id=organisation_id),
        headers=bearer(token),
        params={"limit": 1, "cursor": body["next_cursor"]},
    )
    assert len(second_page.json()["items"]) == 1
    assert second_page.json()["items"][0]["id"] != body["items"][0]["id"]


async def test_org_analysis_list_is_scoped_to_the_organisation(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, org_a = _seed_member(db, subject="researcher-a", role="researcher")
    token_a = await issue_token(client, subject="researcher-a")
    dataset = await _ready_dataset(client, token_a, org_a)
    await _submit(client, token_a, org_a, dataset["id"])

    stranger = create_user(db, subject="stranger")
    org_b = create_organisation(db, name="Stranger Lab")
    create_membership(db, organisation_id=org_b, user_id=stranger, role="owner")
    token_b = await issue_token(client, subject="stranger")

    response = await client.get(ORG_ANALYSES.format(organisation_id=org_b), headers=bearer(token_b))

    assert response.status_code == 200
    assert response.json()["items"] == []
