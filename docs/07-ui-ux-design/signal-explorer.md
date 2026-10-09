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

