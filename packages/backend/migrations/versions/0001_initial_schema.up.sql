-- 0001_initial_schema: up

CREATE TABLE users (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        identity_provider TEXT NOT NULL,
        external_subject TEXT NOT NULL,
        email TEXT,
        display_name TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        disabled_at TIMESTAMPTZ,
        CONSTRAINT ck_users_identity_provider_not_blank
            CHECK (length(btrim(identity_provider)) > 0),
        CONSTRAINT ck_users_external_subject_not_blank
            CHECK (length(btrim(external_subject)) > 0),
        CONSTRAINT uq_users_identity UNIQUE (identity_provider, external_subject)
    );

CREATE TABLE organisations (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        name TEXT NOT NULL,
        slug TEXT,
        status TEXT NOT NULL DEFAULT 'active',
        created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        CONSTRAINT ck_organisations_name_not_blank CHECK (length(btrim(name)) > 0),
        CONSTRAINT ck_organisations_name_length CHECK (length(name) <= 200),
        CONSTRAINT ck_organisations_status CHECK (status IN ('active', 'archived')),
        CONSTRAINT uq_organisations_slug UNIQUE (slug)
    );

CREATE TABLE memberships (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        organisation_id UUID NOT NULL REFERENCES organisations (id) ON DELETE CASCADE,
        user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        role TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        CONSTRAINT ck_memberships_role CHECK (role IN ('owner', 'researcher', 'viewer')),
        CONSTRAINT uq_memberships_organisation_user UNIQUE (organisation_id, user_id)
    );

CREATE INDEX ix_memberships_user_id ON memberships (user_id);

CREATE UNIQUE INDEX uq_memberships_organisation_id_id ON memberships (organisation_id, id);

CREATE TABLE studies (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        organisation_id UUID NOT NULL REFERENCES organisations (id),
        name TEXT NOT NULL,
        description TEXT,
        status TEXT NOT NULL DEFAULT 'active',
        created_by UUID NOT NULL REFERENCES users (id),
        created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        archived_at TIMESTAMPTZ,
        CONSTRAINT ck_studies_name_not_blank CHECK (length(btrim(name)) > 0),
        CONSTRAINT ck_studies_name_length CHECK (length(name) <= 200),
        CONSTRAINT ck_studies_description_length
            CHECK (description IS NULL OR length(description) <= 2000),
        CONSTRAINT ck_studies_status CHECK (status IN ('active', 'archived')),
        CONSTRAINT ck_studies_archived_at
            CHECK ((status = 'archived') = (archived_at IS NOT NULL))
    );

CREATE UNIQUE INDEX uq_studies_organisation_id_id ON studies (organisation_id, id);

CREATE INDEX ix_studies_organisation_created_at
        ON studies (organisation_id, created_at DESC, id DESC);

CREATE INDEX ix_studies_organisation_status_updated_at
        ON studies (organisation_id, status, updated_at DESC, id DESC);

CREATE TABLE datasets (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        organisation_id UUID NOT NULL REFERENCES organisations (id),
        study_id UUID NOT NULL,
        name TEXT NOT NULL,
        description TEXT,
        status TEXT NOT NULL DEFAULT 'pending_upload',
        format TEXT NOT NULL DEFAULT 'npy',
        storage_key TEXT,
        object_version TEXT,
        checksum_sha256 TEXT,
        file_size_bytes BIGINT,
        sampling_frequency_hz DOUBLE PRECISION,
        channel_count INTEGER,
        sample_count BIGINT,
        duration_seconds DOUBLE PRECISION,
        channel_names JSONB,
        channel_units JSONB,
        metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
        validation_report JSONB,
        source_description TEXT,
        uploaded_by UUID NOT NULL REFERENCES users (id),
        created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        validated_at TIMESTAMPTZ,
        archived_at TIMESTAMPTZ,
        CONSTRAINT ck_datasets_name_not_blank CHECK (length(btrim(name)) > 0),
        CONSTRAINT ck_datasets_name_length CHECK (length(name) <= 200),
        CONSTRAINT ck_datasets_status CHECK (
            status IN ('pending_upload', 'validating', 'ready', 'rejected', 'failed', 'archived')
        ),
        CONSTRAINT ck_datasets_format CHECK (format IN ('npy')),
        CONSTRAINT ck_datasets_file_size
            CHECK (file_size_bytes IS NULL OR file_size_bytes >= 0),
        CONSTRAINT ck_datasets_sampling_frequency
            CHECK (sampling_frequency_hz IS NULL OR sampling_frequency_hz > 0),
        CONSTRAINT ck_datasets_channel_count
            CHECK (channel_count IS NULL OR channel_count > 0),
        CONSTRAINT ck_datasets_sample_count
            CHECK (sample_count IS NULL OR sample_count >= 0),
        CONSTRAINT ck_datasets_duration
            CHECK (duration_seconds IS NULL OR duration_seconds >= 0),
        CONSTRAINT ck_datasets_checksum CHECK (
            checksum_sha256 IS NULL OR checksum_sha256 ~ '^[0-9a-f]{64}$'
        ),
        CONSTRAINT ck_datasets_ready_requires_metadata CHECK (
            status <> 'ready' OR (
                storage_key IS NOT NULL
                AND checksum_sha256 IS NOT NULL
                AND file_size_bytes IS NOT NULL
                AND sampling_frequency_hz IS NOT NULL
                AND channel_count IS NOT NULL
                AND sample_count IS NOT NULL
                AND duration_seconds IS NOT NULL
                AND channel_names IS NOT NULL
                AND validated_at IS NOT NULL
            )
        ),
        CONSTRAINT fk_datasets_study_same_organisation
            FOREIGN KEY (organisation_id, study_id)
            REFERENCES studies (organisation_id, id)
    );

CREATE UNIQUE INDEX uq_datasets_organisation_id_id ON datasets (organisation_id, id);

CREATE INDEX ix_datasets_organisation_study_created_at
        ON datasets (organisation_id, study_id, created_at DESC, id DESC);

CREATE INDEX ix_datasets_organisation_status_updated_at
        ON datasets (organisation_id, status, updated_at DESC, id DESC);

CREATE TABLE upload_sessions (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        organisation_id UUID NOT NULL REFERENCES organisations (id),
        dataset_id UUID NOT NULL,
        created_by UUID NOT NULL REFERENCES users (id),
        status TEXT NOT NULL DEFAULT 'created',
        object_key TEXT,
        expected_size_bytes BIGINT,
        expected_checksum_sha256 TEXT,
        expires_at TIMESTAMPTZ NOT NULL,
        finalised_at TIMESTAMPTZ,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        CONSTRAINT ck_upload_sessions_status CHECK (
            status IN (
                'created', 'uploading', 'uploaded', 'finalising',
                'completed', 'failed', 'expired'
            )
        ),
        CONSTRAINT ck_upload_sessions_expected_size
            CHECK (expected_size_bytes IS NULL OR expected_size_bytes >= 0),
        CONSTRAINT ck_upload_sessions_expected_checksum CHECK (
            expected_checksum_sha256 IS NULL OR expected_checksum_sha256 ~ '^[0-9a-f]{64}$'
        ),
        CONSTRAINT fk_upload_sessions_dataset_same_organisation
            FOREIGN KEY (organisation_id, dataset_id)
            REFERENCES datasets (organisation_id, id)
    );

CREATE INDEX ix_upload_sessions_organisation_status_expires_at
        ON upload_sessions (organisation_id, status, expires_at);

CREATE INDEX ix_upload_sessions_organisation_dataset
        ON upload_sessions (organisation_id, dataset_id);

CREATE TABLE analysis_jobs (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        organisation_id UUID NOT NULL REFERENCES organisations (id),
        dataset_id UUID NOT NULL,
        requested_by UUID NOT NULL REFERENCES users (id),
        analysis_type TEXT NOT NULL,
        algorithm_version TEXT NOT NULL,
        parameters JSONB NOT NULL DEFAULT '{}'::jsonb,
        status TEXT NOT NULL DEFAULT 'queued',
        attempt_count INTEGER NOT NULL DEFAULT 0,
        max_attempts INTEGER NOT NULL DEFAULT 3,
        error_code TEXT,
        error_message TEXT,
        idempotency_key TEXT,
        requested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        started_at TIMESTAMPTZ,
        completed_at TIMESTAMPTZ,
        lease_expires_at TIMESTAMPTZ,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        CONSTRAINT ck_analysis_jobs_status CHECK (
            status IN ('queued', 'running', 'succeeded', 'failed', 'cancelled')
        ),
        CONSTRAINT ck_analysis_jobs_analysis_type
            CHECK (analysis_type IN ('descriptive_statistics')),
        CONSTRAINT ck_analysis_jobs_attempt_count CHECK (attempt_count >= 0),
        CONSTRAINT ck_analysis_jobs_max_attempts CHECK (max_attempts > 0),
        CONSTRAINT ck_analysis_jobs_completed_at CHECK (
            (status IN ('succeeded', 'failed', 'cancelled')) = (completed_at IS NOT NULL)
        ),
        CONSTRAINT fk_analysis_jobs_dataset_same_organisation
            FOREIGN KEY (organisation_id, dataset_id)
            REFERENCES datasets (organisation_id, id)
    );

CREATE INDEX ix_analysis_jobs_organisation_requested_at
        ON analysis_jobs (organisation_id, requested_at DESC, id DESC);

CREATE INDEX ix_analysis_jobs_organisation_status_requested_at
        ON analysis_jobs (organisation_id, status, requested_at DESC, id DESC);

CREATE INDEX ix_analysis_jobs_organisation_dataset_requested_at
        ON analysis_jobs (organisation_id, dataset_id, requested_at DESC, id DESC);

CREATE INDEX ix_analysis_jobs_lease_expires_at
        ON analysis_jobs (lease_expires_at) WHERE lease_expires_at IS NOT NULL;

CREATE TABLE analysis_results (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        organisation_id UUID NOT NULL REFERENCES organisations (id),
        job_id UUID NOT NULL REFERENCES analysis_jobs (id),
        dataset_id UUID NOT NULL,
        dataset_checksum_sha256 TEXT,
        algorithm_version TEXT NOT NULL,
        result_schema_version TEXT NOT NULL,
        results JSONB NOT NULL,
        provenance JSONB NOT NULL DEFAULT '{}'::jsonb,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        CONSTRAINT uq_analysis_results_job_id UNIQUE (job_id),
        CONSTRAINT ck_analysis_results_checksum CHECK (
            dataset_checksum_sha256 IS NULL OR dataset_checksum_sha256 ~ '^[0-9a-f]{64}$'
        ),
        CONSTRAINT fk_analysis_results_dataset_same_organisation
            FOREIGN KEY (organisation_id, dataset_id)
            REFERENCES datasets (organisation_id, id)
    );

CREATE INDEX ix_analysis_results_organisation_dataset_created_at
        ON analysis_results (organisation_id, dataset_id, created_at DESC, id DESC);

CREATE TABLE outbox_events (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        organisation_id UUID REFERENCES organisations (id),
        event_type TEXT NOT NULL,
        aggregate_type TEXT NOT NULL,
        aggregate_id UUID NOT NULL,
        payload JSONB NOT NULL DEFAULT '{}'::jsonb,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        published_at TIMESTAMPTZ,
        attempt_count INTEGER NOT NULL DEFAULT 0,
        next_attempt_at TIMESTAMPTZ,
        last_error_code TEXT,
        CONSTRAINT ck_outbox_events_event_type_not_blank
            CHECK (length(btrim(event_type)) > 0),
        CONSTRAINT ck_outbox_events_aggregate_type_not_blank
            CHECK (length(btrim(aggregate_type)) > 0),
        CONSTRAINT ck_outbox_events_attempt_count CHECK (attempt_count >= 0)
    );

CREATE INDEX ix_outbox_events_pending
        ON outbox_events (created_at) WHERE published_at IS NULL;

CREATE TABLE idempotency_keys (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        organisation_id UUID NOT NULL REFERENCES organisations (id),
        user_id UUID NOT NULL REFERENCES users (id),
        operation TEXT NOT NULL,
        key TEXT NOT NULL,
        request_hash TEXT NOT NULL,
        response_status INTEGER,
        response_body JSONB,
        resource_id UUID,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        expires_at TIMESTAMPTZ NOT NULL,
        CONSTRAINT uq_idempotency_keys_scope
            UNIQUE (organisation_id, user_id, operation, key)
    );

CREATE INDEX ix_idempotency_keys_expires_at ON idempotency_keys (expires_at);

CREATE TABLE audit_events (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        organisation_id UUID REFERENCES organisations (id),
        actor_user_id UUID REFERENCES users (id),
        action TEXT NOT NULL,
        resource_type TEXT NOT NULL,
        resource_id UUID,
        request_id TEXT,
        metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        CONSTRAINT ck_audit_events_action_not_blank CHECK (length(btrim(action)) > 0),
        CONSTRAINT ck_audit_events_resource_type_not_blank
            CHECK (length(btrim(resource_type)) > 0)
    );

CREATE INDEX ix_audit_events_organisation_created_at
        ON audit_events (organisation_id, created_at DESC, id DESC);

CREATE INDEX ix_audit_events_resource ON audit_events (resource_type, resource_id);
