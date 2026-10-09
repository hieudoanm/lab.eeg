"""API tests for the study endpoints."""

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

STUDIES = "/api/v1/organisations/{organisation_id}/studies"
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
    client: AsyncClient,
    token: str,
    organisation_id: UUID,
    name: str,
    **extra: object,
) -> dict[str, Any]:
    response = await client.post(
        STUDIES.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={"name": name, **extra},
    )
    assert response.status_code == 201, response.text
    return cast(dict[str, Any], response.json())


async def test_create_study_succeeds_for_researcher(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    response = await client.post(
        STUDIES.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={"name": "  Language EEG Pilot  ", "description": "  Exploratory.  "},
    )
    assert response.status_code == 201, response.text
    body = response.json()
    assert body["name"] == "Language EEG Pilot"
    assert body["description"] == "Exploratory."
    assert body["status"] == "active"
    assert body["archived_at"] is None
    assert body["created_at"].endswith("Z")
    assert "organisation_id" not in body


async def test_create_study_requires_authentication(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="owner", role="owner")
    response = await client.post(
        STUDIES.format(organisation_id=organisation_id), json={"name": "Anonymous"}
    )
    assert response.status_code == 401


async def test_viewer_cannot_create_study(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="viewer", role="viewer")
    token = await issue_token(client, subject="viewer")
    response = await client.post(
        STUDIES.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={"name": "Denied"},
    )
    assert response.status_code == 403
    assert response.json()["error"]["code"] == "insufficient_role"


async def test_create_study_rejects_unknown_fields(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    response = await client.post(
        STUDIES.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={"name": "Study", "status": "archived"},
    )
    assert response.status_code == 422
    assert response.json()["error"]["code"] == "validation_error"


async def test_create_study_rejects_blank_name(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    response = await client.post(
        STUDIES.format(organisation_id=organisation_id),
        headers=bearer(token),
        json={"name": "   "},
    )
    assert response.status_code == 422


async def test_list_studies_returns_empty_page(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    response = await client.get(
        STUDIES.format(organisation_id=organisation_id), headers=bearer(token)
    )
    assert response.status_code == 200
    assert response.json() == {"items": [], "next_cursor": None, "has_more": False}


async def test_list_studies_paginates_deterministically(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study_ids = [
        (await _create_study(client, token, organisation_id, name=f"Study {index}"))["id"]
        for index in range(3)
    ]
    url = STUDIES.format(organisation_id=organisation_id)
    first = await client.get(url, headers=bearer(token), params={"limit": 2})
    assert first.status_code == 200
    first_body = first.json()
    assert len(first_body["items"]) == 2
    assert first_body["has_more"] is True
    assert first_body["next_cursor"] is not None
    second = await client.get(
        url, headers=bearer(token), params={"limit": 2, "cursor": first_body["next_cursor"]}
    )
    assert second.status_code == 200
    second_body = second.json()
    assert len(second_body["items"]) == 1
    assert second_body["has_more"] is False
    assert second_body["next_cursor"] is None
    seen = [item["id"] for item in first_body["items"] + second_body["items"]]
    assert sorted(seen) == sorted(study_ids)


async def test_list_studies_searches_by_name(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    await _create_study(client, token, organisation_id, name="Theta Burst Study")
    await _create_study(client, token, organisation_id, name="Alpha Pilot")
    response = await client.get(
        STUDIES.format(organisation_id=organisation_id),
        headers=bearer(token),
        params={"search": "theta"},
    )
    assert response.status_code == 200
    assert [item["name"] for item in response.json()["items"]] == ["Theta Burst Study"]


async def test_list_studies_search_escapes_wildcards(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    await _create_study(client, token, organisation_id, name="Power Study")
    await _create_study(client, token, organisation_id, name="Other Study")
    response = await client.get(
        STUDIES.format(organisation_id=organisation_id),
        headers=bearer(token),
        params={"search": "%"},
    )
    assert response.status_code == 200
    assert [item["name"] for item in response.json()["items"]] == []


async def test_list_studies_filters_by_status(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id, name="Archived soon")
    url = STUDIES.format(organisation_id=organisation_id)
    await client.post(f"{url}/{study['id']}/archive", headers=bearer(token))
    response = await client.get(url, headers=bearer(token), params={"status": "archived"})
    assert [item["id"] for item in response.json()["items"]] == [study["id"]]


async def test_invalid_cursor_is_rejected(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    response = await client.get(
        STUDIES.format(organisation_id=organisation_id),
        headers=bearer(token),
        params={"cursor": "not-a-real-cursor"},
    )
    assert response.status_code == 422
    assert response.json()["error"]["code"] == "invalid_cursor"


async def test_viewer_can_read_study_details(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="owner", role="owner")
    token = await issue_token(client, subject="owner")
    study = await _create_study(client, token, organisation_id, name="Shared study")
    viewer_id = create_user(db, subject="viewer-read")
    create_membership(db, organisation_id=organisation_id, user_id=viewer_id, role="viewer")
    viewer_token = await issue_token(client, subject="viewer-read")
    response = await client.get(
        f"{STUDIES.format(organisation_id=organisation_id)}/{study['id']}",
        headers=bearer(viewer_token),
    )
    assert response.status_code == 200
    assert response.json()["name"] == "Shared study"


async def test_update_study_is_partial(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(
        client, token, organisation_id, name="Original", description="Keep me"
    )
    url = f"{STUDIES.format(organisation_id=organisation_id)}/{study['id']}"
    response = await client.patch(url, headers=bearer(token), json={"name": "Renamed"})
    assert response.status_code == 200
    body = response.json()
    assert body["name"] == "Renamed"
    assert body["description"] == "Keep me"


async def test_update_study_rejects_null_name(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id, name="Keep")
    response = await client.patch(
        f"{STUDIES.format(organisation_id=organisation_id)}/{study['id']}",
        headers=bearer(token),
        json={"name": None},
    )
    assert response.status_code == 422
    details = response.json()["error"]["details"]
    assert any(
        detail["code"] == "null_not_allowed" and detail["field"] == "name" for detail in details
    )


async def test_update_study_clears_description(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id, name="S", description="Temporary")
    response = await client.patch(
        f"{STUDIES.format(organisation_id=organisation_id)}/{study['id']}",
        headers=bearer(token),
        json={"description": None},
    )
    assert response.status_code == 200
    assert response.json()["description"] is None


async def test_viewer_cannot_update_study(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="owner", role="owner")
    token = await issue_token(client, subject="owner")
    study = await _create_study(client, token, organisation_id, name="No edit")
    viewer_id = create_user(db, subject="viewer-writer")
    create_membership(db, organisation_id=organisation_id, user_id=viewer_id, role="viewer")
    viewer_token = await issue_token(client, subject="viewer-writer")
    response = await client.patch(
        f"{STUDIES.format(organisation_id=organisation_id)}/{study['id']}",
        headers=bearer(viewer_token),
        json={"name": "Hacked"},
    )
    assert response.status_code == 403


async def test_archived_study_is_read_only(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id, name="Archive me")
    url = f"{STUDIES.format(organisation_id=organisation_id)}/{study['id']}"
    await client.post(f"{url}/archive", headers=bearer(token))
    response = await client.patch(url, headers=bearer(token), json={"name": "Renamed"})
    assert response.status_code == 409
    assert response.json()["error"]["code"] == "archived_study"


async def test_archiving_a_study_is_idempotent(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    study = await _create_study(client, token, organisation_id, name="Twice")
    url = f"{STUDIES.format(organisation_id=organisation_id)}/{study['id']}/archive"
    first = await client.post(url, headers=bearer(token))
    assert first.status_code == 200
    assert first.json()["status"] == "archived"
    assert first.json()["archived_at"]
    second = await client.post(url, headers=bearer(token))
    assert second.status_code == 200
    assert second.json()["archived_at"] == first.json()["archived_at"]


async def test_archive_missing_study_returns_404(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, organisation_id = _seed_member(db, subject="researcher", role="researcher")
    token = await issue_token(client, subject="researcher")
    response = await client.post(
        f"{STUDIES.format(organisation_id=organisation_id)}/{MISSING}/archive",
        headers=bearer(token),
    )
    assert response.status_code == 404


async def test_study_is_scoped_to_the_caller_organisation(
    client: AsyncClient, db: psycopg.Connection[dict[str, Any]]
) -> None:
    _, org_a = _seed_member(
        db, subject="researcher-a", role="researcher", organisation_name="Organisation A"
    )
    token_a = await issue_token(client, subject="researcher-a")
    study = await _create_study(client, token_a, org_a, name="Private study")
    _, org_b = _seed_member(
        db, subject="researcher-b", role="researcher", organisation_name="Organisation B"
    )
    token_b = await issue_token(client, subject="researcher-b")
    detail = await client.get(
        f"{STUDIES.format(organisation_id=org_b)}/{study['id']}", headers=bearer(token_b)
    )
    assert detail.status_code == 404
    listing = await client.get(STUDIES.format(organisation_id=org_b), headers=bearer(token_b))
    assert listing.json()["items"] == []
