# Datasets, experiments, and provenance

neuro.lab versions everything it produces:

- **Datasets** — imported recordings and derived data, versioned as they
  change.
- **Experiments** — a named collection of pipelines and runs.
- **Analysis runs** — one execution of a pipeline, with progress, status, and
  cancellation.
- **Artifacts** — outputs of runs (reports, exports, figures), each carrying a
  provenance record: inputs, pipeline version, parameters, and timestamps.

Provenance means an artifact can always be traced back to the exact data and
steps that produced it — the core requirement for reproducible neuroimaging
work. All state lives on the local filesystem; nothing is uploaded anywhere.
