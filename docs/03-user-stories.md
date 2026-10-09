# EEG Lab — User Stories and Acceptance Criteria

## 1. Purpose

This document defines the user goals EEG Lab must support and the conditions under which each workflow is considered complete.

User stories are organised by role and workflow. They complement:

- `01-overview.md` — product vision.
- `02-requirements.md` — functional and non-functional requirements.
- `04-features.md` — feature behaviour.
- `09-design.md` — interface and interaction design.

Acceptance criteria should be used by developers, reviewers, and automated tests. They must describe observable behaviour rather than prescribe unnecessary implementation details.

## 2. User Roles

### Researcher

A researcher works with studies, uploads recordings, explores EEG signals, and runs descriptive analyses.

### Viewer

A viewer can inspect resources for which they have been granted access but cannot perform privileged modifications.

### Organisation Owner

An owner manages organisation membership and access permissions. An owner may also use researcher capabilities.

### Operator

An operator maintains application availability, investigates failures, and monitors background processing. Operational access does not automatically grant permission to read scientific data.

## 3. Authentication and Organisation Access

### US-001: Sign in

**As a** user,  
**I want to** sign in securely,  
**so that** I can access the organisations and studies available to me.

Acceptance criteria:

- Given valid credentials, when I sign in, then I receive an authenticated session.
- Given invalid credentials, when I sign in, then I receive a safe error message.
- Given an expired session, when I request a protected resource, then the application asks me to authenticate again or renews the session through the supported mechanism.
- Given an unauthenticated request to a protected API endpoint, then the API rejects the request.
- Authentication errors do not disclose whether another user's account exists unless the chosen identity provider explicitly permits that behaviour.

Priority: P0.

### US-002: Select an organisation

**As a** user who belongs to multiple organisations,  
**I want to** select my active organisation,  
**so that** I can work with the correct resources.

Acceptance criteria:

- Given several organisation memberships, when I select an organisation, then the application loads resources belonging to that organisation.
- Given no membership in the requested organisation, when I attempt to access it, then the server denies access.
- Changing the active organisation clears or invalidates cached organisation-specific data.
- Organisation switching does not expose data from the previously selected organisation.

Priority: P0.

### US-003: Enforce role permissions

**As an** organisation owner,  
**I want to** control what members can do,  
**so that** access matches their responsibilities.

Acceptance criteria:

- Given a viewer, when they attempt a write operation, then the API rejects it.
- Given a researcher, when they perform an allowed study or dataset operation, then the API permits it.
- Given an unauthorised user, when they call an API directly rather than using the UI, then the same permission rules apply.
- The frontend hides or disables unavailable actions where appropriate, but server-side authorisation remains authoritative.

Priority: P0.

## 4. Study Management

### US-004: Create a study

**As a** researcher,  
**I want to** create a study,  
**so that** I can organise related datasets and analyses.

Acceptance criteria:

- Given a valid study name, when I submit the form, then the study is created in my active organisation.
- Given an empty or invalid name, when I submit the form, then validation explains what needs to change.
- Given a successful creation, then the study appears in the study list.
- Repeated submission caused by a retry must not unintentionally create duplicate studies when the request uses an idempotency key.
- A user cannot create a study in an organisation they do not belong to.

Priority: P0.

### US-005: Browse and search studies

**As a** researcher,  
**I want to** browse and search studies,  
**so that** I can find the work I need.

Acceptance criteria:

- The list displays the study name and relevant summary metadata.
- Search filters results within the active organisation.
- Pagination does not expose resources outside the user's permitted scope.
- An empty organisation displays a helpful empty state.
- A search with no matches displays a distinct no-results state.
- Loading and API failures are distinguishable from an empty list.

Priority: P0.

### US-006: Edit or archive a study

**As a** researcher with appropriate permissions,  
**I want to** update or archive a study,  
**so that** the workspace remains organised.

Acceptance criteria:

- A permitted user can update supported study fields.
- Invalid changes are rejected with actionable feedback.
- A viewer cannot update or archive a study.
- Archiving does not silently delete the underlying datasets, results, or audit history.
- Archived studies cannot accept new analyses unless that behaviour is explicitly supported and documented.

Priority: P0.

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

## 6. Signal Exploration

### US-010: Visualise an EEG recording

**As a** researcher,  
**I want to** view EEG signals over time,  
**so that** I can explore the recording's structure.

Acceptance criteria:

- Given a ready dataset, when I open the signal explorer, then I can view a bounded interval of available channels.
- The horizontal axis represents time.
- The vertical axis represents signal amplitude, with units shown when known.
- Channel labels and the selected time interval are visible.
- The chart does not need to load the entire recording into browser memory.
- An inaccessible dataset cannot be retrieved by changing the request URL or resource identifier.

Priority: P0.

### US-011: Navigate the signal

**As a** researcher,  
**I want to** select channels and navigate time intervals,  
**so that** I can inspect relevant portions of a recording.

Acceptance criteria:

- I can select an available channel or supported group of channels.
- I can request a different time interval within the recording.
- The chart supports the documented zoom, pan, and reset interactions.
- Requests exceeding configured limits are rejected or constrained with clear feedback.
- Display downsampling is identified where applicable.
- Display downsampling does not modify the source recording or silently replace full-resolution analysis inputs.

Priority: P0.

### US-012: Handle unavailable signal data

**As a** researcher,  
**I want to** understand why a signal cannot be displayed,  
**so that** I can take the next appropriate action.

Acceptance criteria:

- A dataset still being validated displays its current status.
- A recording with no available channels shows a useful explanation.
- An invalid time interval is handled safely.
- A failed request does not leave the chart showing misleading stale data as if it were current.
- A retry is available for recoverable network or service failures.

Priority: P0.

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

## 8. Provenance and History

### US-017: Reproduce an analysis

**As a** researcher,  
**I want to** see which data and algorithm produced a result,  
**so that** I can interpret and reproduce the computation.

Acceptance criteria:

- A result identifies its source dataset and version.
- The source checksum or equivalent content identifier is recorded where supported.
- The algorithm version and parameters are preserved.
- The software version and execution timestamps are available where relevant.
- Re-running an analysis creates a traceable execution record.
- A new algorithm version does not silently overwrite the historical result.

Priority: P1.

### US-018: Review analysis history

**As a** researcher,  
**I want to** review previous jobs,  
**so that** I can compare outcomes and investigate failures.

Acceptance criteria:

- The user can view jobs associated with an accessible study or dataset.
- Job status, analysis type, and timestamps are shown.
- Successful jobs link to their results.
- Failed jobs expose safe diagnostic information.
- History respects organisation and resource-level access controls.

Priority: P1.

## 9. Organisation Administration

### US-019: Manage members

**As an** organisation owner,  
**I want to** manage organisation membership,  
**so that** users have appropriate access.

Acceptance criteria:

- An owner can inspect member roles.
- An owner can perform the membership changes supported by the implemented invitation mechanism.
- Non-owners cannot perform owner-only operations.
- The last-owner invariant is preserved.
- Membership changes generate audit events.
- A removed member cannot access protected resources through an existing session after the change takes effect.

Priority: P1.

## 10. Audit and Operations

### US-020: Review important actions

**As an** authorised operator or owner,  
**I want to** trace important changes,  
**so that** I can investigate unexpected behaviour.

Acceptance criteria:

- Supported security-relevant actions generate audit events.
- Each event includes an actor, action, resource, and timestamp where available.
- Audit records do not contain credentials, access tokens, or raw EEG payloads.
- Ordinary users cannot modify audit history through public application endpoints.

Priority: P1.

### US-021: Monitor service health

**As an** operator,  
**I want to** determine whether the application and its dependencies are healthy,  
**so that** I can identify operational problems.

Acceptance criteria:

- The liveness endpoint indicates whether the API process is alive.
- The readiness endpoint reflects the health of essential dependencies.
- Health endpoints do not disclose credentials or connection strings.
- Application and worker logs include enough context to correlate relevant failures.
- Logs avoid unnecessary sensitive scientific data.

Priority: P0.

## 11. Accessibility and Usability

### US-022: Use the application with accessible controls

**As a** user,  
**I want to** navigate the application with accessible controls,  
**so that** I can complete workflows using different input methods.

Acceptance criteria:

- Primary workflows can be navigated with a keyboard.
- Form fields have meaningful labels.
- Validation errors are associated with the relevant fields.
- Status changes are communicated without relying on colour alone.
- Charts provide a useful text-based summary or tabular alternative for essential information.
- Focus remains predictable after dialogs, submissions, and navigation.

Priority: P1.

## 12. Cross-Cutting Acceptance Rules

These rules apply to every story:

1. Authorisation is enforced on the server.
2. Organisation-scoped queries are explicitly scoped and tested.
3. Invalid input is rejected before expensive processing.
4. Errors are actionable but do not expose secrets or internal implementation details.
5. Loading, empty, success, and failure states are implemented where relevant.
6. State-changing operations are safe against accidental duplicate submission.
7. Relevant unit and integration tests are added.
8. Important workflows receive end-to-end coverage.
9. Database changes include migrations.
10. API behaviour is documented and consistent with `07-api-spec.md`.
11. Staged backend and frontend source changes pass the configured Husky pre-commit checks; CI runs the complete quality gates.

## 13. MVP Acceptance Checklist

The MVP is ready for demonstration when all P0 stories are implemented and verified.

- [ ] A user can authenticate.
- [ ] A user can select an authorised organisation.
- [ ] A researcher can create and browse studies.
- [ ] A researcher can upload a supported EEG recording.
- [ ] Invalid uploads are rejected safely.
- [ ] A ready dataset exposes its metadata.
- [ ] A researcher can explore a bounded signal interval.
- [ ] A researcher can submit a descriptive analysis.
- [ ] The analysis runs asynchronously.
- [ ] The user can monitor the job and inspect its result.
- [ ] Failures are handled and can be diagnosed safely.
- [ ] Cross-organisation access is prevented and tested.
- [ ] Health checks, structured logs, and automated tests are available.
- [ ] The workflow can be demonstrated using public or synthetic data without requiring access to private research data.
