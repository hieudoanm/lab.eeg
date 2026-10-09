# EEG Lab — UI/UX Design Specification

## 1. Purpose

This document defines EEG Lab's information architecture, visual identity, page layouts, interaction patterns, accessibility requirements, and responsive behaviour.

The interface should feel like a professional scientific research platform: precise, calm, data-oriented, and easy to navigate. It should demonstrate strong product engineering without distracting visual effects or unnecessary complexity.

The design must support the workflows defined in:

- `01-overview.md`
- `02-requirements.md`
- `04-features.md`
- `03-user-stories.md`

## 2. Design Principles

### 2.1 Clarity before decoration

Every screen should make three things obvious:

1. Where am I?
2. What data am I looking at?
3. What can I do next?

Avoid decorative components that compete with scientific data.

### 2.2 Data density with readable hierarchy

Researchers need to inspect metadata, tables, job states, and charts efficiently.

Use compact layouts for scientific information while preserving readable text, clear labels, and sufficient spacing.

### 2.3 Scientific precision

Display measurement units, timestamps, channel names, processing states, and provenance wherever relevant.

Never imply that an approximate display representation is the original raw recording.

### 2.4 Honest system feedback

Every asynchronous operation must communicate its state. Do not show a success notification before the server confirms success.

### 2.5 Consistent interactions

Use the same patterns for navigation, forms, status badges, dialogs, tables, errors, and confirmations throughout the application.

### 2.6 Security-aware presentation

Do not expose resources, identifiers, metadata, or actions that the current user is not authorised to access.

Frontend visibility is a usability measure, not a substitute for server-side authorisation.

## 3. Visual Direction

### Design concept

**Scientific workstation meets modern research SaaS.**

The interface should resemble a focused research tool rather than a consumer wellness application or a generic administrative dashboard.

Characteristics:

- Neutral backgrounds.
- Dark, high-contrast text.
- Restrained indigo or blue accents.
- Subtle borders and surface separation.
- Monospaced typography for technical identifiers and numerical values where useful.
- Minimal shadows.
- Consistent chart colours and accessible status indicators.
- Clear visual distinction between navigation, content, and data visualisations.

Avoid excessive gradients, glassmorphism, neon effects, animated backgrounds, and oversized marketing-style headings inside the authenticated application.

## 4. Design Tokens

Implement the visual system through reusable design tokens and component-level styling rather than scattered one-off values.

### 4.1 Colour palette

The palette is defined once and rendered through two themes: `lab-light` (the
default) and `lab-dark`. Both are declared in `globals.css` as daisyUI themes and
as `--color-*` semantic tokens. Components reference the token, never a literal,
so the same markup adapts to the active theme.

| Token            | Light       | Dark      | Usage                          |
| ---------------- | ----------- | --------- | ------------------------------ |
| `background`     | `#F6F8FB`   | `#0B0F14` | Application background         |
| `surface`        | `#FFFFFF`   | `#121821` | Cards, tables, dialogs         |
| `surface-muted`  | `#EEF2F7`   | `#1A2230` | Secondary surfaces             |
| `surface-raised` | `#E2E8F0`   | `#212B3A` | Menus, popovers, dropdowns     |
| `overlay-scrim`  | `#17203366` | `#05080B` | Modal and drawer backdrop      |
| `border`         | `#DCE3EC`   | `#263244` | Dividers and outlines          |
| `border-strong`  | `#C3CDD9`   | `#35455C` | Emphasised borders             |
| `text-primary`   | `#172033`   | `#E6EDF5` | Main text                      |
| `text-secondary` | `#5B6578`   | `#9BA9BC` | Supporting text                |
| `text-muted`     | `#8A94A6`   | `#6B7A90` | Disabled / de-emphasised text  |
| `accent`         | `#4059C7`   | `#4C8DFF` | Primary actions and links      |
| `accent-hover`   | `#3146A5`   | `#6BA1FF` | Hover state                    |
| `accent-muted`   | `#E8ECFB`   | `#1C3355` | Accent tint backgrounds        |
| `success`        | `#18794E`   | `#3FB950` | Successful operations          |
| `success-muted`  | `#E1F4EC`   | `#16301F` | Success tint backgrounds       |
| `warning`        | `#986500`   | `#D9A441` | Warnings                       |
| `warning-muted`  | `#FBF0DE`   | `#33290F` | Warning tint backgrounds       |
| `danger`         | `#B42318`   | `#F0554D` | Errors and destructive actions |
| `danger-muted`   | `#FBEAE9`   | `#3A1C1A` | Error tint backgrounds         |
| `info`           | `#175CD3`   | `#58A6FF` | Informational feedback         |
| `info-muted`     | `#E7EEFC`   | `#16283F` | Info tint backgrounds          |
| `neutral-state`  | `#8A94A6`   | `#5B6578` | Pending / queued / cancelled   |
| `canvas`         | `#EEF2F7`   | `#05080B` | Signal trace canvas background |
| `focus-ring`     | `#4059C7`   | `#7FB2FF` | Keyboard focus outline         |

The EEG trace palette used by the Signal Explorer and time-series charts:

| Token     | Light     | Dark      | Usage                  |
| --------- | --------- | --------- | ---------------------- |
| `trace-1` | `#4059C7` | `#4C8DFF` | Channel trace colour 1 |
| `trace-2` | `#175CD3` | `#58A6FF` | Channel trace colour 2 |
| `trace-3` | `#18794E` | `#3FB950` | Channel trace colour 3 |
| `trace-4` | `#986500` | `#D9A441` | Channel trace colour 4 |
| `trace-5` | `#B42318` | `#F0554D` | Channel trace colour 5 |
| `trace-6` | `#7C3AED` | `#B07CFF` | Channel trace colour 6 |

Status colours must always be accompanied by text or another non-colour indicator.

### 4.2 Typography

Use a clean sans-serif font for the interface.

Recommended system stack:

`Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif`

Use a monospace stack for hashes, identifiers, timestamps where appropriate, and technical values:

`ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace`

Suggested hierarchy:

| Element             | Suggested size |
| ------------------- | -------------: |
| Page title          |       28–32 px |
| Section heading     |       20–24 px |
| Card heading        |       16–18 px |
| Body text           |       14–16 px |
| Secondary metadata  |       12–14 px |
| Dense table content |       13–14 px |

Maintain a comfortable line height for paragraphs and documentation.

### 4.3 Spacing

Use a consistent 4 px base spacing scale.

Typical spacing values:

- 4 px: icon-to-label spacing.
- 8 px: compact component spacing.
- 12 px: form field and metadata spacing.
- 16 px: standard component padding.
- 24 px: card and section spacing.
- 32 px: major section separation.
- 40–48 px: large page-level separation.

Avoid inconsistent arbitrary spacing values.

### 4.4 Borders and elevation

- Use subtle one-pixel borders to distinguish surfaces.
- Reserve shadows for overlays and components that genuinely need elevation.
- Use a consistent corner radius, typically 8–12 px for cards and panels.
- Avoid nested cards unless each level has a clear semantic purpose.

### 4.5 Component layer (daisyUI)

The palette is implemented as two daisyUI v5 themes declared in `globals.css`: `lab-light` (default, `color-scheme: light`) and `lab-dark` (`prefersdark: true`, `color-scheme: dark`). Every component inherits consistent colours, radii, and control sizing from the active theme.

The theme is switched at runtime by setting the `data-theme` attribute on the root element. A `ThemeProvider`/`ThemeToggle` pair (in `components/atoms`) exposes the active theme and a toggle; the choice is persisted in `localStorage` under the key `lab-theme`, and with no stored value the page follows the user's `prefers-color-scheme`. An inline boot script applies the stored theme before first paint so there is no flash of the wrong theme.

Mapping from design tokens (§4.1) to daisyUI theme variables (the mapping is identical for both themes; only the underlying token values change):

| daisyUI variable       | EEG Lab token                                    |
| ---------------------- | ------------------------------------------------ |
| `--color-base-100`     | `surface`                                        |
| `--color-base-200`     | `surface-muted`                                  |
| `--color-base-300`     | `border`                                         |
| `--color-base-content` | `text-primary`                                   |
| `--color-primary`      | `accent`                                         |
| `--color-secondary`    | `text-secondary`                                 |
| `--color-neutral`      | `text-primary` (light) / `surface-raised` (dark) |
| `--color-info`         | `info`                                           |
| `--color-success`      | `success`                                        |
| `--color-warning`      | `warning`                                        |
| `--color-error`        | `danger`                                         |

Radii map `--radius-field` to input/field radius, `--radius-box` to card and panel radius, and `--radius-selector` to pill-shaped controls. Elevation and noise are disabled, matching §4.4.

Prefer daisyUI component classes — `btn`, `input`, `select`, `textarea`, `file-input`, `badge`, `card`, `table`, `steps`, `progress`, `alert`, `menu`, `dropdown`, and `loading` — inside the reusable components in §20 instead of bespoke utility stacks.

## 5. Application Layout

### 5.1 Desktop layout

Use a persistent left navigation sidebar and a main content area.

```text
┌─────────────────────────────────────────────────────────────┐
│ EEG Lab                         Organisation  User Menu    │
├───────────────┬─────────────────────────────────────────────┤
│               │ Breadcrumbs                                  │
│ Dashboard     │                                             │
│ Studies       │ Page Title                   Primary Action  │
│ Datasets      │ Supporting description                      │
│ Analyses      │                                             │
│               │ Main content                                │
│               │ Tables / Forms / Charts / Details            │
│               │                                             │
│ Settings      │                                             │
└───────────────┴─────────────────────────────────────────────┘
```

The precise navigation items may vary by role and implementation scope.

### 5.2 Sidebar

The sidebar should include:

- EEG Lab brand.
- Dashboard.
- Studies.
- Analyses or job history.
- Organisation settings, where permitted.
- User menu or account access.

Datasets should normally be discoverable through studies as well as any global dataset listing that is implemented.

The sidebar should:

- Highlight the active destination.
- Use recognisable icons consistently.
- Support keyboard navigation.
- Collapse on smaller screens.
- Avoid showing links to unavailable or unimplemented features.

### 5.3 Top bar

The top bar should include:

- Current organisation selector.
- User account menu.
- Breadcrumbs where useful.
- Optional global navigation controls.

Organisation switching must clearly indicate the active organisation and must refresh organisation-scoped content safely.

### 5.4 Content area

The content area should use a consistent maximum width for ordinary forms and overview pages.

Data-heavy views, especially signal exploration, may use more of the available horizontal space.

Avoid horizontal overflow on common desktop widths.

## 6. Information Architecture

Recommended route structure:

```text
/
├── login
├── app
│   ├── dashboard
│   ├── studies
│   │   ├── new
│   │   └── [studyId]
│   │       ├── settings
│   │       └── datasets
│   │           ├── upload
│   │           └── [datasetId]
│   │               ├── overview
│   │               ├── signals
│   │               └── analyses
│   ├── analyses
│   │   └── [jobId]
│   │       └── result
│   └── settings
│       ├── profile
│       └── organisation
└── not-found
```

Adapt this structure to the chosen frontend router. It is a logical route map, not a requirement to create empty pages for every path.

### Navigation rules

- Protected pages require authentication.
- Resource pages require server-verified access.
- Breadcrumbs should use human-readable resource names where available.
- Refreshing a deep link must preserve the intended destination if the user remains authorised.
- Unknown routes display a proper not-found page.
- Unauthorised and nonexistent resources should be handled consistently to avoid leaking resource existence.

## 7. Dashboard

### Purpose

Provide a concise overview of the active organisation's research activity.

### Suggested content

- Welcome heading.
- Active organisation.
- Recent studies.
- Recent analysis jobs.
- Counts of studies, datasets, and jobs when available.
- Shortcuts to create a study or open recent work.

### Layout

Use a compact summary row followed by recent activity sections.

Avoid charts that provide little actionable information. A small number of useful summaries is better than a dashboard filled with decorative graphs.

### States

**First-time user:** explain how to create a study.

**Existing user:** show recent studies and job activity.

**Loading:** display placeholders or progress indicators.

**Error:** explain which content could not be loaded and offer a safe retry.

## 8. Study List

### Purpose

Help users find and organise research studies.

### Layout

The page should include:

- Page title and concise description.
- Search field.
- Create-study action for authorised users.
- Table or responsive list of studies.
- Pagination when needed.

### Table columns

Recommended columns:

- Study name.
- Dataset count.
- Status.
- Last updated.
- Actions.

On narrow screens, use a compact list or hide lower-priority metadata rather than forcing the entire table into a tiny viewport.

### Interactions

- Clicking a study opens its detail page.
- Search updates the result set predictably.
- Destructive or significant actions require appropriate confirmation.
- Keyboard users can reach row actions.
- Empty and no-results states provide distinct guidance.

## 9. Study Detail

### Purpose

Act as the central workspace for a study.

### Header

Display:

- Study name.
- Status.
- Description, if available.
- Last updated timestamp.
- Actions based on permissions.

### Main sections

Use tabs or clearly separated sections for:

1. Overview.
2. Datasets.
3. Analyses.
4. Settings, if supported.

### Overview content

Show study metadata and a compact activity summary.

### Dataset section

Show dataset names, validation states, recording metadata, and actions.

### Analysis section

Show recent and historical jobs associated with the study.

Avoid deeply nested tabs. The user should be able to navigate directly to a dataset or analysis.

## 10. Dataset Upload Page

### Purpose

Make uploading and validating a recording understandable and recoverable.

### Layout

Use a focused form rather than a large dashboard.

Suggested sections:

1. Dataset details.
2. File selection.
3. Recording metadata.
4. Upload and validation progress.
5. Final outcome.

### File selection

The file input should clearly display:

- Selected filename.
- File size.
- Supported format.
- Configured size limit.
- Removal or replacement action before submission.

Do not rely exclusively on drag-and-drop. A standard accessible file picker must remain available.

### Validation feedback

Show progress through clearly labelled stages, such as:

```text
Upload → Validation → Ready
```

If validation fails, display:

- A concise explanation.
- Relevant field or file information.
- A corrective action.
- A retry option only when retrying is meaningful.

Do not show a success state until the backend confirms the dataset is ready.

## 11. Dataset Overview Page

### Purpose

Provide an authoritative summary of a recording and its availability.

### Header

Display:

- Dataset name.
- Validation status.
- Parent study.
- Primary action to explore signals.
- Analysis action when permitted.

### Metadata section

Present metadata as labelled values rather than an unstructured paragraph.

Include where available:

- Format.
- Channel count.
- Sampling frequency.
- Sample count.
- Duration.
- Measurement units.
- Checksum.
- Source and provenance.
- Upload timestamp.

### Status behaviour

If the dataset is validating, disable actions that require a ready recording and explain why.

If validation fails, show the safe failure reason and available recovery options.

## 12. Signal Explorer

### Purpose

Provide a focused workspace for exploring EEG time series.

The signal explorer is the most visually important scientific screen in the MVP.

### Desktop layout

```text
┌──────────────────────────────────────────────────────────────┐
│ Study / Dataset breadcrumb                                   │
│ Dataset name                         Ready                   │
├──────────────────────────────────────────────────────────────┤
│ Channels              │ Signal view                          │
│                       │                                      │
│ [x] Fp1               │  ┌────────────────────────────────┐  │
│ [x] Fp2               │  │                                │  │
│ [ ] C3                │  │       EEG time-series chart     │  │
│ [ ] C4                │  │                                │  │
│                       │  └────────────────────────────────┘  │
│                       │                                      │
│                       │ Time range: [start] — [end]          │
│                       │ [Reset]              [Load interval]  │
├───────────────────────┴──────────────────────────────────────┤
│ Recording metadata / selected interval information           │
└──────────────────────────────────────────────────────────────┘
```

This is a conceptual layout. The implementation must derive channel names from the recording metadata rather than hardcode the example names.

### Chart requirements

- Use Plotly.js or another documented scientific plotting library.
- Display a clearly labelled time axis.
- Display signal amplitude and units when known.
- Support the documented zoom, pan, and reset interactions.
- Make the selected time interval visible.
- Use consistent channel labelling.
- Avoid excessive chart decorations.
- Provide loading and error feedback without leaving stale data misleadingly visible.

### Channel selection

The channel panel should:

- Display available channel names.
- Support selecting and deselecting channels.
- Indicate the number of selected channels.
- Respect configured channel limits.
- Remain usable when the recording contains many channels.

### Time range selection

Use explicit start and end values with validation.

The interface should prevent or explain:

- Negative start times.
- End times before start times.
- Intervals beyond recording duration.
- Intervals exceeding configured limits.

Zooming within the loaded interval should not imply that additional source data has been fetched. Make the distinction between chart zoom and loading a new interval clear.

### Performance

- Request bounded signal data from the API.
- Show a loading state when fetching a new interval.
- Avoid re-rendering unrelated application areas when chart state changes.
- Keep chart updates responsive for the documented dataset limits.
- Identify display downsampling when used.
- Never alter the original recording to improve chart performance.

### Accessibility

Charts must not be the sole means of communicating essential information.

Provide a concise textual summary or accessible tabular representation for the selected interval where practical. Ensure surrounding controls can be operated by keyboard.

## 13. Analysis Submission and Job Detail

### Analysis submission

Use a simple form or dialog showing:

- Dataset name.
- Analysis type.
- Parameters supported by the selected analysis.
- A concise description of the computation.
- Submit and cancel actions.

Do not expose parameters that the backend does not support.

### Job detail layout

Display:

- Job identifier.
- Status.
- Analysis type.
- Dataset.
- Submitted timestamp.
- Started and completed timestamps when available.
- Algorithm version.
- Parameters.
- Safe error information or a link to the result.

### Status presentation

Use a text label and consistent visual indicator for each state:

| State     | Presentation                         |
| --------- | ------------------------------------ |
| Queued    | Neutral label and queued text        |
| Running   | Progress indicator and running text  |
| Succeeded | Success indicator and succeeded text |
| Failed    | Error indicator and failed text      |
| Cancelled | Neutral label and cancelled text     |

A running indicator must not imply a known percentage of completion unless the backend actually provides meaningful progress information.

## 14. Analysis Result Page

### Purpose

Present descriptive statistics in a way that is easy to inspect and trace back to its source.

### Header

Display:

- Analysis type.
- Job status.
- Dataset name.
- Execution timestamp.
- Algorithm version.

### Results table

Recommended columns:

- Channel.
- Mean.
- Standard deviation.
- Minimum.
- Maximum.
- Peak-to-peak range.
- Valid sample count.

Use clear numerical formatting without discarding meaningful precision. Display units where known.

### Result metadata

Include a collapsible or secondary section for:

- Dataset version or checksum.
- Analysis parameters.
- Software version.
- Execution metadata.
- Provenance information.

### Interpretation

Use neutral language.

The interface may explain what a statistic measures, but it must not infer disease, diagnosis, treatment effectiveness, or neurological meaning unsupported by the computation.

## 15. Forms, Dialogs, and Feedback

### Forms

Every form should have:

- A clear title.
- Labels for every field.
- Required-field indicators where appropriate.
- Inline validation.
- Disabled or progress state during submission.
- A clear success or failure outcome.

Preserve entered values after recoverable failures.

### Dialogs

Use dialogs for short, focused actions such as confirming an archive operation.

Do not place complex multi-step scientific workflows inside small dialogs when a full page would be clearer.

### Notifications

Use non-blocking notifications for routine success or failure messages.

For important validation failures, show the explanation in the page or form as well.

Notifications should:

- Be announced appropriately to assistive technologies.
- Remain readable.
- Avoid conveying essential information through colour alone.
- Never claim success before the server confirms it.

### Destructive actions

Clearly identify the affected resource and the consequence of the operation.

Use a confirmation step when the operation is destructive or difficult to reverse. Prefer archive or reversible operations over permanent deletion where appropriate.

## 16. Responsive Behaviour

### Desktop: 1200 px and wider

- Persistent sidebar.
- Multi-column metadata layouts where useful.
- Full-width signal chart.
- Tables with their important columns visible.

### Tablet: 768–1199 px

- Collapsible or narrower sidebar.
- Reduced card columns.
- Responsive metadata layouts.
- Signal chart takes priority over secondary panels.

### Mobile: below 768 px

- Collapsible navigation.
- Single-column forms and detail layouts.
- Compact tables or stacked resource cards.
- Chart controls remain accessible.
- Channel selection can use a drawer or collapsible panel.
- No essential workflow depends on hover.

These breakpoints are starting points, not rigid rules. Adjust them based on actual content and usability testing.

## 17. Accessibility

Target WCAG 2.2 Level AA where applicable.

Requirements include:

- Sufficient text and interface contrast.
- Keyboard access to primary workflows.
- Visible focus indicators.
- Meaningful labels and accessible names.
- Semantic heading hierarchy.
- Accessible form errors.
- Status indicators that do not depend on colour alone.
- Respect for reduced-motion preferences.
- Accessible alternatives for essential chart information.
- Usable layouts at increased zoom.

Do not sacrifice accessibility to achieve a visually minimal interface.

## 18. Loading, Empty, and Error States

Every data-driven screen must distinguish between the following states.

### Loading

Use a progress indicator or skeleton. Avoid flashing an empty state before data arrives.

### Empty

Explain why the collection is empty and provide a next step if the user has permission to take one.

### No results

Explain that the current filter or search found no matching resources.

### Validation error

Identify what needs correcting and preserve relevant form input.

### Permission error

Explain that the resource or operation is unavailable without exposing protected information.

### Network failure

Provide a safe retry mechanism where possible. Do not repeat non-idempotent mutations automatically unless their idempotency is guaranteed.

### Unexpected server error

Display a friendly message and a request identifier when available. Do not expose stack traces, SQL errors, credentials, or internal service details.

## 19. Motion and Micro-interactions

Motion should clarify state transitions rather than decorate the interface.

Appropriate examples:

- Subtle hover and focus feedback.
- Short transitions for expanding panels.
- A progress indicator during uploads.
- A visible state change when an analysis completes.

Avoid:

- Constantly moving backgrounds.
- Unnecessary animated chart effects.
- Long transitions that slow down work.
- Animations that ignore reduced-motion preferences.

## 20. Component Library

Implement reusable components for repeated patterns on top of the daisyUI primitives described in §4.5.

Suggested components:

- `AppShell`
- `Sidebar`
- `TopBar`
- `Breadcrumbs`
- `PageHeader`
- `Button`
- `TextField`
- `SelectField`
- `FilePicker`
- `StatusBadge`
- `DataTable`
- `EmptyState`
- `ErrorState`
- `LoadingState`
- `ConfirmDialog`
- `MetadataGrid`
- `ChannelSelector`
- `TimeRangeSelector`
- `SignalChart`
- `JobStatusPanel`
- `AnalysisResultsTable`
- `ToastProvider`

Use the actual naming conventions of the selected framework. Do not create abstractions that have only one trivial use without a clear benefit.

## 20.1 Atomic Design Structure

The component library follows the **Atomic Design** methodology to ensure maintainability, reusability, and clear separation of concerns:

### 20.1.1 Atoms — Basic building blocks

Foundational, indivisible UI elements that cannot be broken down further without losing their function:

| Component                                      | Purpose                                                             |
| ---------------------------------------------- | ------------------------------------------------------------------- |
| `Button`                                       | Primary, secondary, ghost, and outline actions                      |
| `TextField`                                    | Single-line text input with label and validation                    |
| `TextareaField`                                | Multi-line text input                                               |
| `SelectField`                                  | Single-select dropdown                                              |
| `Card`                                         | Content container with optional title and actions                   |
| `DataTable`                                    | Sortable, paginated data table                                      |
| `MetadataGrid`                                 | Key–value grid for structured metadata display                      |
| `StatusBadge`                                  | Semantic status indicator (success, warning, danger, info, neutral) |
| `ToastProvider` / `useToast`                   | Non-blocking notification system                                    |
| `LoadingState`                                 | Spinner with optional label                                         |
| `EmptyState`                                   | Empty collection with action                                        |
| `ErrorState`                                   | Error display with request ID and retry                             |
| `NoResultsState`                               | Filtered search with no matches                                     |
| `Breadcrumbs`                                  | Hierarchical navigation trail                                       |
| `SignOutButton`                                | Authentication sign-out action                                      |
| `ArchiveStudyButton`                           | Study archival action with confirmation                             |
| `TimeSeriesChart`                              | Plotly-based EEG time-series visualisation                          |
| `StatisticsTable` / `StatisticsSummary`        | Per-channel and aggregate statistics                                |
| `AnalysisStatusBadge` / `AnalysisStatusDetail` | Analysis job status presentation                                    |

### 20.1.2 Molecules — Combinations of atoms

Composed components that combine multiple atoms to form a distinct functional unit:

| Component              | Composition                                           | Purpose                            |
| ---------------------- | ----------------------------------------------------- | ---------------------------------- |
| `LoginForm`            | `TextField`, `Button`, `Toast`                        | Email-based authentication         |
| `DatasetForm`          | `TextField`, `TextareaField`, `SelectField`, `Button` | Dataset metadata CRUD              |
| `StudyForm`            | `TextField`, `TextareaField`, `Button`                | Study CRUD                         |
| `DatasetUploadForm`    | `FilePicker`, progress stages, `Button`               | Multi-step upload + validation     |
| `PageHeader`           | Title, description, `Breadcrumbs`, action             | Consistent page headers            |
| `TopBar`               | Organisation switcher, user menu, `SignOutButton`     | Application header                 |
| `OrganisationSwitcher` | `SelectField`, organisation list                      | Tenant context switcher            |
| `Sidebar`              | Navigation links, active state, mobile drawer         | Primary navigation                 |
| `ChannelSelector`      | `SelectField`, channel list                           | EEG channel picker                 |
| `ChartContainer`       | `TimeSeriesChart`, `StatisticsTable`, toggle          | Signal view with data table        |
| `AnalysisActions`      | `Button`, `AnalysisStatusBadge`, job list             | Analysis submission and management |

### 20.1.3 Organisms — Complex components

Sophisticated components that combine molecules and atoms into complete application sections:

| Component             | Composition                                                                 | Purpose                  |
| --------------------- | --------------------------------------------------------------------------- | ------------------------ |
| `AppShell`            | `Sidebar`, `TopBar`, main content area, skip link                           | Application layout shell |
| `SignalVisualisation` | `ChannelSelector`, `TimeSeriesChart`, `ChartContainer`, time range controls | Full signal explorer     |

### 20.1.4 Directory structure

```text
src/components/
├── atoms/
│   ├── button.tsx
│   ├── text-field.tsx
│   ├── ... (17 atom components)
│   ├── time-series-chart.tsx
│   ├── statistics-table.tsx
│   ├── analysis-status.tsx
│   ├── index.ts
│   └── __tests__/
├── molecules/
│   ├── login-form.tsx
│   ├── dataset-form.tsx
│   ├── ... (9 molecule components)
│   ├── channel-selector.tsx
│   ├── chart-container.tsx
│   ├── index.ts
│   └── __tests__/
├── organisms/
│   ├── app-shell.tsx
│   ├── signal-visualisation.tsx
│   ├── index.ts
│   └── __tests__/
└── index.ts  # Barrel export: atoms → molecules → organisms
```

Each layer exports a barrel file (`index.ts`) so consumers import from the layer root (`@/components/atoms`, `@/components/molecules`, `@/components/organisms`) rather than individual files. The top-level `src/components/index.ts` re-exports all three layers.

**Benefits:**

- Clear ownership: atoms are framework-agnostic primitives; molecules express domain logic; organisms assemble full features.
- Testability: atoms and molecules can be unit-tested in isolation; organisms test composition.
- Reusability: atoms and molecules are reused across organisms and pages.
- Scalability: new components fit naturally into one layer without restructuring.

## 21. Frontend State and Interaction Rules

- Use URL state for navigable context where appropriate, such as selected study or dataset.
- Keep temporary chart interactions local to the explorer unless sharing or persistence is explicitly required.
- Clear organisation-scoped caches when the active organisation changes.
- Cancel obsolete requests when users rapidly change datasets or time intervals.
- Prevent stale responses from overwriting newer selections.
- Disable duplicate submissions while an operation is in progress.
- Do not use optimistic success for uploads, analysis jobs, or other operations whose success requires server confirmation.
- Keep server-owned resource state distinct from local presentation state.

## 22. Design Deliverables

The implementation should include:

- A reusable design-token system.
- A consistent application shell.
- Responsive navigation.
- Study list and detail pages.
- Dataset upload and detail pages.
- Interactive signal explorer.
- Analysis submission and status views.
- Analysis results table.
- Reusable loading, empty, and error states.
- Accessible forms and controls.
- Consistent status indicators.
- A not-found page.
- A clear authentication experience.

## 23. Definition of Done

The design is considered implemented when:

1. Primary workflows follow a consistent visual system.
2. Users can navigate between studies, datasets, and analyses without confusion.
3. The signal explorer makes channels, time ranges, units, and loading states clear.
4. Upload and job statuses reflect actual backend state.
5. All primary actions have appropriate loading, success, and failure feedback.
6. Layouts remain usable on desktop, tablet, and mobile.
7. Keyboard navigation and focus management work across primary workflows.
8. Essential chart information has an accessible alternative.
9. No controls advertise unsupported functionality.
10. The interface does not make unsupported medical or scientific claims.
11. Changed frontend source files pass the Husky pre-commit ESLint and Prettier checks; full frontend tests and type checks remain separate gates.
