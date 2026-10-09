# 0007 — Dual Lab Themes (`lab-light` / `lab-dark`)

- Status: Accepted
- Date: 2026-10-13
- Phase: Frontend styling

## Context

[0006](0006-daisyui-component-layer.md) mapped the EEG Lab palette to a single
daisyUI theme (`eeg-lab`) that was light-only. As the three Lab applications
(EEG, MEG, MRI) were unified, the theme needed to be:

- identical across the three repositories, so documentation, mockups, and
  components stay in sync; and
- available in both a light and a dark variant, selectable by the user.

MEG and MRI had historically shipped a dark-only interface while EEG shipped a
light-only one, so a single theme could not represent all three.

## Decision

- **Ship two daisyUI v5 themes in every Lab application**, both declared in
  `globals.css` via `@plugin 'daisyui/theme'`:
  - `lab-light` — `default: true`, `prefersdark: false`, `color-scheme: light`.
  - `lab-dark` — `default: false`, `prefersdark: true`, `color-scheme: dark`.
- **Keep the `@theme` token layer.** The shared `--color-*` semantic tokens
  (`background`, `surface`, `surface-muted`, `surface-raised`, `overlay-scrim`,
  `text-primary`, `text-secondary`, `text-muted`, `border`, `border-strong`,
  `accent`, `accent-hover`, `accent-muted`, `success`, `warning`, `danger`,
  `info` and their `-muted` variants, `neutral-state`, `canvas`, `focus-ring`)
  are declared once with the light values and re-declared under
  `[data-theme='lab-dark']` with the dark values, so a single utility class
  adapts to the active theme.
- **Switch themes at runtime with a persisted toggle.** A `ThemeProvider` /
  `ThemeToggle` pair (in `components/atoms`) sets the `data-theme` attribute on
  the root element and persists the choice in `localStorage` under the key
  `lab-theme`. With no stored value the page follows the user's
  `prefers-color-scheme`. An inline boot script applies the stored theme before
  first paint to avoid a flash of the wrong theme.
- **Rename domain tokens to kebab-case** and keep a small set of domain-specific
  tokens per application (`trace-1`…`trace-6` for EEG/MEG; `atlas`, `mask`,
  `annotation`, `measurement` for MRI).

The token values and the token-to-daisyUI mapping are documented in
`docs/09-design.md` §4.

## Consequences

- `docs/09-design.md` §4 and the design READMEs describe both themes and the
  toggle.
- The standalone HTML mockups ship the same two themes and an in-page switcher,
  and the screenshot tool renders each screen in both themes into
  `images/light/` and `images/dark/` (same file name in each folder).
- `09-design.md` §4 identifies `--color-foreground` as `--color-text-primary`
  and `--color-muted-foreground` as `--color-text-secondary`.
- New components must reference tokens, not literals, so they remain
  theme-agnostic.
