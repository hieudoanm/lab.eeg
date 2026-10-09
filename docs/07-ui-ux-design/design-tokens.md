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

