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
