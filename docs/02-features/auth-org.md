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

