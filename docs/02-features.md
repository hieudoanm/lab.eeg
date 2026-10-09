# EEG Lab — Feature Specifications

## 1. Purpose

This document describes how EEG Lab features should behave from the user's perspective and how their behaviour should be implemented.

It complements:

- `01-overview.md` — product vision and scope.
- `02-requirements.md` — functional and non-functional requirements.
- `03-user-stories.md` — user goals and acceptance scenarios.
- `09-design.md` — interface and interaction design.
- `05-architecture.md` — system boundaries and technical responsibilities.

Features should be delivered in vertical slices: a user should be able to complete a useful workflow through the frontend, API, database, and relevant background services before the next major workflow is started.

## 2. Feature Priorities

| Priority     | Meaning                                            |
| ------------ | -------------------------------------------------- |
| P0           | Required for a usable, secure MVP                  |
| P1           | Important for a convincing portfolio demonstration |
| P2           | Valuable enhancement after the MVP is stable       |
| Out of scope | Not part of the initial product                    |

## 3. Authentication and Organisation Context

**Priority:** P0

### Description

Users authenticate and work within an organisation. Organisation context determines which studies, datasets, jobs, and results they may access.

### User experience

- The application provides a clear sign-in experience.
- After authentication, the user sees their available organisations.
- The user selects an organisation or returns to the last valid selection.
- The application displays the active organisation consistently.
- If a session expires, the user receives a clear recovery path.
- If the user has no organisation membership, the application explains what they can do next.

### Behaviour

1. Authenticate the user.
2. Retrieve the user's identity and memberships.
3. Select an organisation for which membership has been verified.
4. Load organisation-scoped resources.
5. Include the organisation context in relevant API requests.
6. Independently verify membership and permissions on the server.

### Edge cases

- Invalid or expired session.
- Removed organisation membership.
- Organisation deleted or deactivated.
- User belongs to several organisations.
- User attempts to alter an organisation identifier in an API request.
- Authentication provider unavailable.

### Completion criteria

- Users cannot access private application resources without authentication.
- Changing the active organisation changes the visible resource scope.
- Direct API requests cannot bypass membership checks.
- Authentication errors are handled without exposing sensitive details.

## 4. Study Management

**Priority:** P0

### Description

A study is the primary container for organising datasets and their associated analyses.

### User experience

The study list should show:

- Study name.
- Short description, when present.
- Number of datasets.
- Study status.
- Last updated timestamp.
- Actions available to the current user.

Users can search, paginate, create, inspect, edit, and archive studies according to their permissions.

### Create a study

Required fields:

- Name.

Optional fields:

- Description.

The form must prevent accidental duplicate submissions and provide inline validation.

On success, the application should navigate to the new study or display a clear confirmation.

### Study details

The detail page should contain:

- Study overview.
- Dataset list.
- Analysis history.
- Relevant provenance information.
- Actions appropriate to the user's role.

### Archive behaviour

Archiving a study must not silently destroy its datasets, analysis results, or audit history.

Archived studies should be read-only by default. Any exceptions must be explicitly documented.

### Edge cases

- Empty study list.
- Search returns no matches.
- Invalid or overly long name.
- Network failure during save.
- Concurrent edits.
- User loses permission while viewing a study.
- Study is archived while another operation is in progress.

## 5. Dataset Upload and Validation

**Priority:** P0

### Description

Researchers upload EEG recordings and metadata into a study. EEG Lab validates the recording before making it available for exploration or analysis.

### Supported initial format

The initial implementation should choose and document one practical format:

- NumPy `.npy` recording plus a JSON metadata manifest; or
- EDF, using a maintained EEG-compatible parser.

Do not claim support for a format until its parser, validation, and tests are implemented.

### Upload workflow

<list>
  <list-item>**Select a study.** The user opens a study and starts a new dataset upload.</list-item>
  <list-item>**Enter metadata.** The user provides a dataset name, source information, sampling frequency, channel names, units, and other required metadata.</list-item>
  <list-item>**Select the recording.** The interface checks basic file constraints before uploading.</list-item>
  <list-item>**Upload the file.** The file is transferred to private storage, directly or through a controlled backend endpoint.</list-item>
  <list-item>**Validate the recording.** The backend verifies the file, parses the recording, and checks consistency with the manifest.</list-item>
  <list-item>**Display the result.** The interface shows whether validation succeeded, failed, or is still in progress.</list-item>
</list>

### Validation behaviour

Validation should check:

- Supported format.
- File size.
- File integrity and checksum.
- Expected dimensions.
- Numeric data validity.
- Sampling frequency.
- Channel metadata.
- Resource limits.
- Compatibility between the recording and its manifest.

Validation errors must identify the problem without revealing internal paths, stack traces, or implementation details.

### Dataset lifecycle

`pending_upload` → `validating` → `ready`

Alternative outcomes:

- `rejected`: input violates the supported format or validation rules.
- `failed`: an operational error prevents validation from completing.

A failed validation must not be treated as a successful upload.

### Dataset details

The dataset page should display:

- Name and recording identifier.
- Source and provenance.
- File format and checksum.
- Sampling frequency.
- Channel count and names.
- Number of samples and recording duration.
- Units, where known.
- Validation status.
- Upload and validation timestamps.
- Available visualisation and analysis actions.

### Edge cases

- Interrupted upload.
- Duplicate upload request.
- Missing metadata.
- Unsupported file.
- Malformed manifest.
- Checksum mismatch.
- File content does not match its extension.
- Non-finite signal values.
- Extremely long recording.
- Too many channels.
- Storage service unavailable.
- User loses permission during upload.
- Uploaded object exists but database finalisation fails.

### Important design rule

Never load an unbounded recording entirely into API memory merely to validate it. Apply file-size and resource limits, and use chunked or bounded processing where appropriate.

## 6. Interactive Signal Visualisation

**Priority:** P0

### Description

Researchers inspect a bounded section of an EEG recording to understand its structure and identify regions of interest.

The visualisation is an exploration tool, not a clinical interpretation system.

### Initial chart

The chart should display:

- Time on the horizontal axis.
- Signal amplitude on the vertical axis.
- Channel labels.
- Measurement units when known.
- Sampling frequency.
- Selected time interval.
- Any display downsampling indicator.

Plotly.js is the preferred initial visualisation library.

### User interactions

Users should be able to:

- Select a channel or a limited group of channels.
- Choose a time interval.
- Zoom and pan.
- Reset the chart view.
- Inspect signal values where practical.
- Navigate to another interval.
- Understand whether displayed points have been downsampled.

### Data retrieval

The client requests only the selected interval and channels.

The server must enforce maximum interval duration, channel count, and response size. Client-side limits alone are insufficient.

### Downsampling

When a recording contains too many points for efficient display, the server or a dedicated processing layer may produce a display-oriented representation.

The implementation must:

- Keep the original recording unchanged.
- Make the downsampling method explicit.
- Preserve the correct time axis.
- Avoid presenting display samples as the full-resolution source.
- Keep descriptive analyses independent of display downsampling.

### Empty and error states

The interface should explain when:

- The dataset is still validating.
- No channels are available.
- The selected interval is invalid.
- The requested interval exceeds the allowed limit.
- The recording cannot be retrieved.
- The server returns no signal data.

### Completion criteria

An authorised user can select a ready dataset, request a bounded interval, and inspect its channels without loading the complete recording into the browser.

## 7. Asynchronous Analysis Jobs

**Priority:** P0

### Description

Researchers submit descriptive analyses that execute in a background worker rather than blocking an API request.

### Initial analysis

Implement one well-tested descriptive analysis containing:

- Mean.
- Standard deviation.
- Minimum.
- Maximum.
- Peak-to-peak range.
- Valid sample count.

Results should be computed per channel.

The implementation must document the standard deviation convention and how invalid or non-finite values are treated.

### Submission workflow

1. The user opens a ready dataset.
2. The user selects the supported analysis.
3. The frontend submits an analysis request.
4. The API validates permissions, parameters, and dataset state.
5. The API creates a job record and schedules work.
6. The API returns the job resource.
7. A worker executes the computation.
8. The worker persists the result and final job state.
9. The frontend refreshes or polls the job status.
10. The user opens the result.

### Job states

| State       | Meaning                                                                |
| ----------- | ---------------------------------------------------------------------- |
| `queued`    | Accepted but not yet executing                                         |
| `running`   | A worker is processing the job                                         |
| `succeeded` | Computation and result persistence completed                           |
| `failed`    | Processing failed and will not be retried automatically                |
| `cancelled` | Processing was cancelled according to the supported cancellation rules |

State transitions must be validated and persisted.

### Result view

The result page should show:

- Analysis type.
- Job status.
- Dataset version.
- Algorithm version.
- Parameters.
- Submission and completion timestamps.
- Per-channel statistics.
- Safe failure information, if applicable.

### Reliability requirements

- Job submission must be idempotent under the documented conditions.
- Workers must tolerate duplicate delivery.
- Transient failures must use bounded retries.
- Permanent input errors must not be retried indefinitely.
- A worker crash must not leave jobs permanently ambiguous.
- Results must not be persisted as successful before computation and validation finish.
- Database state and queue publication must be coordinated to avoid silently losing accepted jobs.

A transactional outbox is a suitable approach if the chosen queue cannot participate in the database transaction.

### Edge cases

- Dataset is archived before the job starts.
- User loses access after submission.
- Worker crashes during computation.
- Queue temporarily becomes unavailable.
- Same request is submitted repeatedly.
- Result persistence fails after computation.
- Job exceeds its execution limit.
- Frontend closes while the worker continues.

## 8. Job Monitoring and Analysis History

**Priority:** P1

### Description

Researchers need to inspect recent and historical analyses for a study or dataset.

### Behaviour

The interface should support:

- Listing jobs.
- Filtering by status.
- Viewing job details.
- Opening successful results.
- Inspecting safe error details for failed jobs.
- Retrying eligible failures.
- Distinguishing a new run from a previous result.

### Retry semantics

A retry should be an explicit, auditable operation.

The system must distinguish:

- Retrying an existing job, where supported.
- Creating a new job from the same dataset and parameters.
- Creating a job against a newer dataset version.

Historical results must remain attributable to the original inputs and algorithm version.

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

## 10. Organisation Membership Management

**Priority:** P1

### Description

Organisation owners manage access to their organisation.

### Initial capabilities

- List members.
- Inspect each member's role.
- Invite or add a member through the chosen identity mechanism.
- Change roles where permitted.
- Remove membership.
- Preserve the last-owner invariant.

If invitations are outside the initial MVP, the UI should not display non-functional invitation controls.

### Security

Every membership change must be authorised server-side and recorded as an audit event.

## 11. Audit Trail

**Priority:** P1

### Description

The application records security-relevant actions to support troubleshooting and accountability.

### Initial events

- Study created, updated, or archived.
- Membership created, changed, or removed.
- Dataset uploaded.
- Dataset validation completed or failed.
- Analysis submitted, retried, completed, or failed.
- Access-control changes.

### Behaviour

Audit events must include the actor, organisation, action, resource, and timestamp when available.

Audit data must not contain access tokens, passwords, signed URLs, or unnecessary raw scientific data.

## 12. Operational Health

**Priority:** P0

### Description

Developers and operators need to determine whether the API and its dependencies are functioning.

### Health endpoints

- `/healthz`: confirms that the API process is alive.
- `/readyz`: reports whether essential dependencies are ready.

Health endpoints must not reveal secrets or connection strings.

### Logs

Use structured logs with:

- Timestamp.
- Severity.
- Request or job identifier.
- Relevant organisation identifier when safe.
- Operation name.
- Outcome.
- Error classification.

Avoid logging raw EEG arrays, credentials, signed URLs, or sensitive payloads.

## 13. User Interface States

**Priority:** P0

Every primary screen must implement the states relevant to its workflow.

### Loading

Show progress or a skeleton for asynchronous requests. Avoid misleading empty screens while data is still loading.

### Empty

Explain why there is no content and offer an appropriate next action.

### Success

Confirm the completed operation and show the resulting resource or next step.

### Validation error

Identify the invalid field or file property and explain how to fix it.

### Permission error

Explain that the requested action is unavailable without exposing another tenant's protected resources.

### Network error

Allow safe retry where appropriate and preserve user-entered form data when possible.

### Unexpected server error

Show a safe message and request identifier where available. Do not expose internal stack traces.

## 14. Deferred Features

The following features must not delay the initial MVP:

- Real-time EEG hardware integration.
- Imagined-speech decoding.
- Brain-computer interface control.
- Machine-learning model training at scale.
- Clinical diagnosis or treatment recommendations.
- Collaborative real-time editing.
- Billing and subscription management.
- Advanced statistical modelling.
- Automatic scientific interpretation.
- Public sharing of raw recordings.
- Arbitrary user-uploaded executable analysis code.

These features may be considered only after the core workflow, security controls, and reproducibility requirements are stable.

## 15. Definition of Feature Completion

A feature is complete only when:

1. Its intended behaviour is documented.
2. The frontend and backend implement the agreed contract.
3. Authorisation is enforced on the server.
4. Validation and failure states are handled.
5. Relevant automated tests pass.
6. Database migrations are included where required.
7. Logs and operational behaviour are adequate.
8. User-facing documentation is updated.
9. No critical security requirement is knowingly violated.
10. The feature has been exercised through an end-to-end workflow where practical.
11. Changed backend and frontend source files pass the repository's Husky pre-commit checks.
