## 5. Dataset Upload

### US-007: Upload an EEG recording

**As a** researcher,  
**I want to** upload an EEG recording to a study,  
**so that** I can inspect and analyse it.

Acceptance criteria:

- Given a study I can modify, when I upload a supported file with valid metadata, then the system accepts it for validation.
- The application shows the upload and validation status.
- The dataset becomes available for exploration only after validation succeeds.
- The source recording is stored privately.
- The system records relevant provenance and integrity information, including a checksum where supported.
- A viewer cannot upload a dataset.
- Uploading to a study outside my permitted organisation is rejected.

Priority: P0.

### US-008: Handle invalid uploads

**As a** researcher,  
**I want to** understand why an upload failed,  
**so that** I can correct the problem.

Acceptance criteria:

- Unsupported formats produce a clear error.
- Missing or inconsistent metadata is identified.
- Corrupt files are rejected.
- Files exceeding configured limits are rejected safely.
- A failed upload does not create a dataset that appears ready for analysis.
- Internal filesystem paths, stack traces, and secrets are not shown to the user.
- The failure is logged with a request or operation identifier where appropriate.

Priority: P0.

### US-009: Inspect dataset metadata

**As a** researcher,  
**I want to** inspect recording metadata,  
**so that** I understand the data before analysis.

Acceptance criteria:

- The dataset page displays available recording metadata.
- Channel names, sample count, sampling frequency, duration, and units are displayed when known.
- Unknown metadata is explicitly represented as unknown rather than fabricated.
- Dataset status determines which actions are available.
- A dataset still being validated cannot be used for analysis.

Priority: P0.

