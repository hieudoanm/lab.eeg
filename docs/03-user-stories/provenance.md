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
