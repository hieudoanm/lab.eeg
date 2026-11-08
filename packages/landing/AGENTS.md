# AGENTS.md — packages/landing

Static marketing page for Neuro Lab. **No build step, no package.json, no
JavaScript.** The entire site is flat files — `index.html` + `styles.css` +
assets at the root, no subdirectories.

## Layout

```
landing/
├── index.html    # Single-page markup: nav, hero, features, demo, how-it-works, install, CTA, footer
├── 404.html      # GitHub Pages 404 page, reuses styles.css
├── styles.css    # All styling — design tokens + components, hand-written
├── logo.svg      # Brand mark (light theme)
├── favicon.svg   # Browser favicon
├── og.svg        # Social share image
├── robots.txt    # Allows all crawlers, points at sitemap.xml
├── sitemap.xml   # Single URL: https://hieudoanm.github.io/neuro.lab/
├── humans.txt    # Project/team credits
└── README.md, AGENTS.md
```

## Rules

- **Keep it flat.** No subdirectories — all assets live at the root. Do not
  add frameworks, bundlers, npm packages, or `<script>` tags. If
  interactivity is ever needed, prefer CSS-only solutions.
- **Relative paths only.** Assets are referenced as `logo.svg`,
  `styles.css` — never absolute `/...` paths, so the page works under any
  base path (GitHub Pages serves it at `/neuro.lab/`).
- **One light theme only** — `#ffffff` surfaces, `#f9fafb` alt sections,
  `#111827` text, `#e5e7eb` borders (same palette family as the llm.rs,
  neo.bank, and transcribers landings). Terminal mockups (`.terminal`)
  intentionally stay dark. Do not add theme variants, `prefers-color-scheme`
  handling, or `*-light`/`*-dark` asset duplicates.
- **Design tokens live in `:root`** at the top of `styles.css` — colors,
  radii, fonts, spacing, shadows. Reuse them; don't hardcode new hex values
  in components.
- **Unslop design rules** apply (see `notes/ai-unslop-design.md` in the
  hieudoanm notes repo): no filler badges, monospace only for code/terminals,
  consistent icon family (24px viewBox, 1.5px stroke, `currentColor`), teal
  `--accent` for CTAs/accents only, real pipeline JSON in demos — no mock
  illustrations, `tabular-nums` for numbers, `text-wrap: balance` on headings.
- No gradients. Accent is teal (`--accent` family); reserve it for CTAs,
  icons, and small highlights.
- Demo pipeline JSON must reflect real node types/params — keep it in sync
  with `crates/neuro-eeg/src/pipeline.rs`, `crates/neuro-meg/src/pipeline.rs`,
  and `crates/neuro-mri/src/operations.rs`.

## Verify

There is no test/build tooling. Sanity-check by serving the directory and
opening `index.html`:

```bash
python3 -m http.server 8080 -d packages/landing
```
