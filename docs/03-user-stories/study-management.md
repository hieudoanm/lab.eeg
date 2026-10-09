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

