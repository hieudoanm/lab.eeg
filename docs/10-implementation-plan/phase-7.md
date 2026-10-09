- Job state persists across API restarts.
- Duplicate requests do not create duplicate logical jobs.
- A duplicate worker delivery does not corrupt results.
- Results are committed only after successful validation.
- A failed analysis does not return fabricated results.
- Worker crashes are observable and recoverable.
- Provenance includes dataset identity or checksum, algorithm version, parameters, and result schema version.

## 11. Phase 7 — Scientific Visualisation

### Objectives

Present recording metadata and numerical results clearly.

### Tasks

- [x] Integrate Plotly.js.
- [x] Create a channel-selection control.
- [x] Display a time-series view.
- [x] Calculate the time axis from sampling frequency.
- [x] Display channel metadata.
- [x] Display analysis summaries.
- [x] Display per-channel statistics.
- [x] Add analysis status indicators.
- [x] Add accessible alternatives to charts.
- [x] Add chart loading and failure states.
- [x] Test with recordings of different lengths.

### Visualisation requirements

- Use the actual sampling frequency when constructing time axes.
- Label axes and units where known.
- State when units are unknown.
- Avoid plotting millions of samples directly in the browser.
- Introduce server-side or worker-side downsampling when necessary.
- Do not describe descriptive statistics as clinical findings.
- Do not infer diagnoses or cognitive traits from the initial output.

### Acceptance criteria

- Users can inspect synthetic recordings.
- Channel selection works.
- Time axes reflect the supplied sampling frequency.
- Analysis values match backend results.
- Long recordings remain usable under documented limits.
- Charts have meaningful labels and accessible summaries.

