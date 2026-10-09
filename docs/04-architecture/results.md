## 11. Result Persistence and Reproducibility

An analysis result must be linked to the exact input and computation that produced it.

Record:

- Source dataset and version.
- Source checksum or stable content identifier.
- Analysis type.
- Algorithm version.
- Parameters.
- Result schema version.
- Execution timestamps.
- Relevant software version.

Persist the result and successful terminal job state consistently.

If result persistence fails, the job must not appear successful.

If a result artefact is stored in object storage, account for the lack of a shared transaction between PostgreSQL and object storage. Use a staged write and finalisation process, or another documented recovery mechanism.

Historical results should not be silently overwritten by later runs.

