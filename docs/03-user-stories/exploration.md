## 6. Signal Exploration

### US-010: Visualise an EEG recording

**As a** researcher,  
**I want to** view EEG signals over time,  
**so that** I can explore the recording's structure.

Acceptance criteria:

- Given a ready dataset, when I open the signal explorer, then I can view a bounded interval of available channels.
- The horizontal axis represents time.
- The vertical axis represents signal amplitude, with units shown when known.
- Channel labels and the selected time interval are visible.
- The chart does not need to load the entire recording into browser memory.
- An inaccessible dataset cannot be retrieved by changing the request URL or resource identifier.

Priority: P0.

### US-011: Navigate the signal

**As a** researcher,  
**I want to** select channels and navigate time intervals,  
**so that** I can inspect relevant portions of a recording.

Acceptance criteria:

- I can select an available channel or supported group of channels.
- I can request a different time interval within the recording.
- The chart supports the documented zoom, pan, and reset interactions.
- Requests exceeding configured limits are rejected or constrained with clear feedback.
- Display downsampling is identified where applicable.
- Display downsampling does not modify the source recording or silently replace full-resolution analysis inputs.

Priority: P0.

### US-012: Handle unavailable signal data

**As a** researcher,  
**I want to** understand why a signal cannot be displayed,  
**so that** I can take the next appropriate action.

Acceptance criteria:

- A dataset still being validated displays its current status.
- A recording with no available channels shows a useful explanation.
- An invalid time interval is handled safely.
- A failed request does not leave the chart showing misleading stale data as if it were current.
- A retry is available for recoverable network or service failures.

Priority: P0.

