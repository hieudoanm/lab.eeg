"""Integration tests for database constraints and tenant referential integrity."""

from __future__ import annotations

from typing import Any

import psycopg
import pytest

pytestmark = pytest.mark.integration


def _user(db: psycopg.Connection[dict[str, Any]], subject: str = "subject-1") -> str:
    row = db.execute(
        "INSERT INTO users (identity_provider, external_subject) VALUES ('test', %s) RETURNING id",
        (subject,),
    ).fetchone()
    assert row is not None
    return str(row["id"])


def _organisation(db: psycopg.Connection[dict[str, Any]], name: str = "Org") -> str:
    row = db.execute(
        "INSERT INTO organisations (name) VALUES (%s) RETURNING id", (name,)
    ).fetchone()
    assert row is not None
    return str(row["id"])


def _study(db: psycopg.Connection[dict[str, Any]], organisation_id: str, user_id: str) -> str:
    row = db.execute(
        "INSERT INTO studies (organisation_id, name, created_by) "
        "VALUES (%s, 'Study', %s) RETURNING id",
        (organisation_id, user_id),
    ).fetchone()
    assert row is not None
    return str(row["id"])


def _dataset(
    db: psycopg.Connection[dict[str, Any]],
    organisation_id: str,
    study_id: str,
    user_id: str,
) -> str:
    row = db.execute(
        "INSERT INTO datasets (organisation_id, study_id, name, uploaded_by) "
        "VALUES (%s, %s, 'Dataset', %s) RETURNING id",
        (organisation_id, study_id, user_id),
    ).fetchone()
    assert row is not None
    return str(row["id"])


def test_membership_is_unique_per_organisation_and_user(
    db: psycopg.Connection[dict[str, Any]],
) -> None:
    user_id = _user(db)
    organisation_id = _organisation(db)
    db.execute(
        "INSERT INTO memberships (organisation_id, user_id, role) VALUES (%s, %s, 'owner')",
        (organisation_id, user_id),
    )
    with pytest.raises(psycopg.errors.UniqueViolation):
        db.execute(
            "INSERT INTO memberships (organisation_id, user_id, role) VALUES (%s, %s, 'viewer')",
            (organisation_id, user_id),
        )


def test_membership_role_must_be_supported(db: psycopg.Connection[dict[str, Any]]) -> None:
    user_id = _user(db)
    organisation_id = _organisation(db)
    with pytest.raises(psycopg.errors.CheckViolation):
        db.execute(
            "INSERT INTO memberships (organisation_id, user_id, role) VALUES (%s, %s, 'admin')",
            (organisation_id, user_id),
        )


def test_dataset_cannot_reference_another_organisation_study(
    db: psycopg.Connection[dict[str, Any]],
) -> None:
    owner = _user(db, "owner")
    organisation_a = _organisation(db, "Org A")
    organisation_b = _organisation(db, "Org B")
    study_a = _study(db, organisation_a, owner)

    with pytest.raises(psycopg.errors.ForeignKeyViolation):
        _dataset(db, organisation_b, study_a, owner)


def test_dataset_ready_requires_full_recording_metadata(
    db: psycopg.Connection[dict[str, Any]],
) -> None:
    owner = _user(db)
    organisation_id = _organisation(db)
    study_id = _study(db, organisation_id, owner)
    dataset_id = _dataset(db, organisation_id, study_id, owner)

    with pytest.raises(psycopg.errors.CheckViolation):
        db.execute("UPDATE datasets SET status = 'ready' WHERE id = %s", (dataset_id,))


def test_dataset_checksum_must_be_sha256_hex(
    db: psycopg.Connection[dict[str, Any]],
) -> None:
    owner = _user(db)
    organisation_id = _organisation(db)
    study_id = _study(db, organisation_id, owner)
    dataset_id = _dataset(db, organisation_id, study_id, owner)

    with pytest.raises(psycopg.errors.CheckViolation):
        db.execute(
            "UPDATE datasets SET checksum_sha256 = 'not-a-checksum' WHERE id = %s",
            (dataset_id,),
        )


def test_study_archive_requires_timestamp(db: psycopg.Connection[dict[str, Any]]) -> None:
    owner = _user(db)
    organisation_id = _organisation(db)
    with pytest.raises(psycopg.errors.CheckViolation):
        db.execute(
            "INSERT INTO studies (organisation_id, name, created_by, status) "
            "VALUES (%s, 'Archived', %s, 'archived')",
            (organisation_id, owner),
        )


def test_analysis_job_terminal_state_requires_completion_time(
    db: psycopg.Connection[dict[str, Any]],
) -> None:
    owner = _user(db)
    organisation_id = _organisation(db)
    study_id = _study(db, organisation_id, owner)
    dataset_id = _dataset(db, organisation_id, study_id, owner)

    with pytest.raises(psycopg.errors.CheckViolation):
        db.execute(
            "INSERT INTO analysis_jobs "
            "(organisation_id, dataset_id, requested_by, analysis_type, "
            " algorithm_version, status) "
            "VALUES (%s, %s, %s, 'descriptive_statistics', '1.0.0', 'succeeded')",
            (organisation_id, dataset_id, owner),
        )


def test_analysis_job_non_terminal_state_rejects_completion_time(
    db: psycopg.Connection[dict[str, Any]],
) -> None:
    owner = _user(db)
    organisation_id = _organisation(db)
    study_id = _study(db, organisation_id, owner)
    dataset_id = _dataset(db, organisation_id, study_id, owner)

    with pytest.raises(psycopg.errors.CheckViolation):
        db.execute(
            "INSERT INTO analysis_jobs "
            "(organisation_id, dataset_id, requested_by, analysis_type, "
            " algorithm_version, status, completed_at) "
            "VALUES (%s, %s, %s, 'descriptive_statistics', '1.0.0', 'queued', now())",
            (organisation_id, dataset_id, owner),
        )


def test_analysis_job_rejects_unsupported_type(
    db: psycopg.Connection[dict[str, Any]],
) -> None:
    owner = _user(db)
    organisation_id = _organisation(db)
    study_id = _study(db, organisation_id, owner)
    dataset_id = _dataset(db, organisation_id, study_id, owner)

    with pytest.raises(psycopg.errors.CheckViolation):
        db.execute(
            "INSERT INTO analysis_jobs "
            "(organisation_id, dataset_id, requested_by, analysis_type, algorithm_version) "
            "VALUES (%s, %s, %s, 'deep_learning', '1.0.0')",
            (organisation_id, dataset_id, owner),
        )


def test_idempotency_key_scope_is_unique(db: psycopg.Connection[dict[str, Any]]) -> None:
    user_id = _user(db)
    organisation_id = _organisation(db)
    statement = (
        "INSERT INTO idempotency_keys "
        "(organisation_id, user_id, operation, key, request_hash, expires_at) "
        "VALUES (%s, %s, 'analysis.submit', 'key-1', 'hash', now() + interval '1 day')"
    )
    db.execute(statement, (organisation_id, user_id))
    with pytest.raises(psycopg.errors.UniqueViolation):
        db.execute(statement, (organisation_id, user_id))


def test_analysis_result_is_unique_per_job(db: psycopg.Connection[dict[str, Any]]) -> None:
    owner = _user(db)
    organisation_id = _organisation(db)
    study_id = _study(db, organisation_id, owner)
    dataset_id = _dataset(db, organisation_id, study_id, owner)
    job = db.execute(
        "INSERT INTO analysis_jobs "
        "(organisation_id, dataset_id, requested_by, analysis_type, algorithm_version, "
        " status, completed_at) "
        "VALUES (%s, %s, %s, 'descriptive_statistics', '1.0.0', 'succeeded', now()) "
        "RETURNING id",
        (organisation_id, dataset_id, owner),
    ).fetchone()
    assert job is not None
    job_id = job["id"]

    statement = (
        "INSERT INTO analysis_results "
        "(organisation_id, job_id, dataset_id, algorithm_version, "
        " result_schema_version, results) "
        "VALUES (%s, %s, %s, '1.0.0', '1.0.0', '{\"channels\": []}'::jsonb)"
    )
    db.execute(statement, (organisation_id, job_id, dataset_id))
    with pytest.raises(psycopg.errors.UniqueViolation):
        db.execute(statement, (organisation_id, job_id, dataset_id))
