## 9. Provenance and Reproducibility

**Priority:** P1

### Description

EEG Lab should make it possible to understand which data and algorithm produced a result.

### Required provenance

Each successful result should identify:

- Source dataset and version.
- Dataset checksum or stable content identifier.
- Analysis type.
- Algorithm version.
- Analysis parameters.
- Execution timestamp.
- Relevant software version.
- Result schema version.

### Behaviour

- Source data must not be silently overwritten.
- Re-running an analysis must create a traceable execution record.
- Algorithm changes must be reflected in the version identifier.
- Results from different algorithm versions must not be silently merged.
- Provenance must be available through the UI and API.

### Scientific integrity

EEG Lab must not claim that a descriptive statistic demonstrates a neurological condition, treatment effect, or clinical outcome.

