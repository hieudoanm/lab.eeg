-- 0002_add_upload_sessions_manifest: down

ALTER TABLE upload_sessions DROP COLUMN IF EXISTS manifest;
