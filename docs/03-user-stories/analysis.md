## 7. Analysis Jobs

### US-013: Submit a descriptive analysis

**As a** researcher,  
**I want to** run descriptive statistics on a dataset,  
**so that** I can summarise the recording quantitatively.

Acceptance criteria:

- Given a ready dataset and permission to analyse it, when I submit a supported analysis, then the API creates a job.
- The submission returns a job identifier and initial state.
- The computation runs asynchronously rather than blocking the request until completion.
- The job records the dataset version, analysis type, and relevant parameters.
- Repeated requests with the same idempotency key do not create duplicate logical jobs within the documented idempotency scope.
- Invalid parameters and inaccessible datasets are rejected before execution.

Priority: P0.

### US-014: Monitor analysis progress

**As a** researcher,  
**I want to** see the status of an analysis,  
**so that** I know whether it has completed.

Acceptance criteria:

- The interface distinguishes queued, running, succeeded, failed, and cancelled states.
- The displayed status is refreshed through the supported polling or notification mechanism.
- The UI does not display a failed or incomplete job as successful.
- Temporary status-fetch failures are communicated without losing the job identifier.
- A worker crash or retry does not create contradictory terminal states.

Priority: P0.

### US-015: Inspect analysis results

**As a** researcher,  
**I want to** inspect completed descriptive statistics,  
**so that** I can understand the numerical output.

Acceptance criteria:

- Successful results display the supported statistics for each channel.
- The result identifies the source dataset and analysis version.
- The result includes the relevant parameters and execution metadata.
- Failed jobs do not display fabricated or partial values as a completed result.
- Results are accessible only to authorised users in the correct organisation.
- The interface describes the output as descriptive analysis and does not make unsupported clinical claims.

Priority: P0.

### US-016: Recover from an analysis failure

**As a** researcher,  
**I want to** understand and recover from a failed analysis,  
**so that** I can continue my work.

Acceptance criteria:

- A failed job displays a safe, actionable explanation when available.
- Transient infrastructure errors can be retried according to the retry policy.
- Invalid input is not retried indefinitely.
- A retry is traceable to the relevant previous execution.
- The application does not mark a job successful until its result has been persisted successfully.
- Repeated worker delivery does not silently duplicate or corrupt results.

Priority: P0.

