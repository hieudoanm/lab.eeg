-- 0001_initial_schema: down

DROP TABLE IF EXISTS audit_events;

DROP TABLE IF EXISTS idempotency_keys;

DROP TABLE IF EXISTS outbox_events;

DROP TABLE IF EXISTS analysis_results;

DROP TABLE IF EXISTS analysis_jobs;

DROP TABLE IF EXISTS upload_sessions;

DROP TABLE IF EXISTS datasets;

DROP TABLE IF EXISTS studies;

DROP TABLE IF EXISTS memberships;

DROP TABLE IF EXISTS organisations;

DROP TABLE IF EXISTS users;
