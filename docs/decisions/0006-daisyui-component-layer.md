# 0006 — daisyUI Component Layer

- Status: Superseded by [0007](0007-dual-lab-themes.md)
- Date: 2026-10-10
- Phase: Frontend styling

## Context

`docs/09-design.md` defines the EEG Lab palette, spacing, radii, and border
rules as design tokens, and `docs/08-tech-stack.md` §4.4 requires a centralised
token system with a small accessible component layer. The Tailwind v4 build
already exposed the tokens as `@theme` utilities, but every control (buttons,
inputs, badges, cards, tables, steps, progress, alerts, menus, dropdowns) was
hand-assembled from raw utilities.

Hand-rolled controls drift: focus rings, disabled states, sizing, and colour
contrast were repeated inconsistently, and the duplicated class strings made
the component tests brittle. Reproducing an accessible control set from scratch
also conflicts with the guidance not to build a full custom design system before
the main workflows work.

## Decision

- **Adopt daisyUI v5 as the component layer** on top of Tailwind CSS v4. It is a
  development-time Tailwind plugin, so it adds no runtime JavaScript or bundle
  weight.
- **Map EEG Lab tokens to one daisyUI theme** named `eeg-lab`, declared with
  `@plugin 'daisyui/theme'` in `globals.css`. `--color-base-100/200/300`,
  `--color-base-content`, `--color-primary`, `--color-info/success/warning/error`,
  `--radius-field/box/selector`, and the size variables are bound to the values
  in `docs/09-design.md` §4.1. The theme is light-only, with elevation and noise
  disabled to match §4.4.
- **Keep the existing `@theme` tokens.** Both the EEG Lab utility names
  (`bg-base-100`, `text-base-content`, `border-base-300`, …) remain available so
  components can mix semantic utilities with daisyUI components without a second
  colour vocabulary.
- **Wrap daisyUI primitives in the components of `docs/09-design.md` §20.**
  `Button`, `TextField`, `SelectField`, `TextareaField`, `StatusBadge`, `Card`,
  `DataTable`, `LoadingState`, `ErrorState`, and the toast layer keep their
  public props and forward to daisyUI classes, so callers are unaffected.
- **Keep accessibility hooks.** `aria-*` attributes, roles, and the disabled
  behaviour asserted by tests are preserved on the wrapper components.

## Consequences

- `docs/08-tech-stack.md` records daisyUI in the styling row and §4.4, and
  `docs/09-design.md` gains §4.5 documenting the token-to-theme mapping.
- `daisyui` is added as a frontend dev dependency; the production bundle and
  runtime are unchanged.
- Component tests assert daisyUI class names (for example `btn`, `badge-error`,
  `alert-*`) alongside behaviour, so the styling contract is covered.
- Future controls should prefer daisyUI primitives over bespoke utility stacks;
  deviations belong in the wrapper components, not in page code.
