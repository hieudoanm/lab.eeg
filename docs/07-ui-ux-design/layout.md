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

