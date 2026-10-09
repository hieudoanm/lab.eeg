-- 0002_add_upload_sessions_manifest: up

ALTER TABLE upload_sessions
        ADD COLUMN manifest JSONB NOT NULL DEFAULT '{}'::jsonb;
