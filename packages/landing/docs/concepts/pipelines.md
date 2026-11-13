# Pipelines

Processing in neuro.lab is expressed as **reproducible JSON pipeline chains** —
an ordered list of steps that can be filtered, artifact-rejected, epoched, and
more, then re-run identically on new data.

## Properties

- **Serializable** — a pipeline is plain JSON, so it can be stored with an
  experiment, diffed, and shared.
- **Resumable** — long-running steps checkpoint, so an interrupted run picks up
  where it left off.
- **Provenance-aware** — every run records which pipeline version, inputs, and
  parameters produced its artifacts.

Pipelines run inside the desktop app's background-analysis machinery with
progress reporting and cancellation, and the same steps are reachable from the
`neuro` CLI for scripting.
