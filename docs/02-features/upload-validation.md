## 6. Interactive Signal Visualisation

**Priority:** P0

### Description

Researchers inspect a bounded section of an EEG recording to understand its structure and identify regions of interest.

The visualisation is an exploration tool, not a clinical interpretation system.

### Initial chart

The chart should display:

- Time on the horizontal axis.
- Signal amplitude on the vertical axis.
- Channel labels.
- Measurement units when known.
- Sampling frequency.
- Selected time interval.
- Any display downsampling indicator.

Plotly.js is the preferred initial visualisation library.

### User interactions

Users should be able to:

- Select a channel or a limited group of channels.
- Choose a time interval.
- Zoom and pan.
- Reset the chart view.
- Inspect signal values where practical.
- Navigate to another interval.
- Understand whether displayed points have been downsampled.

### Data retrieval

The client requests only the selected interval and channels.

The server must enforce maximum interval duration, channel count, and response size. Client-side limits alone are insufficient.

### Downsampling

When a recording contains too many points for efficient display, the server or a dedicated processing layer may produce a display-oriented representation.

The implementation must:

- Keep the original recording unchanged.
- Make the downsampling method explicit.
- Preserve the correct time axis.
- Avoid presenting display samples as the full-resolution source.
- Keep descriptive analyses independent of display downsampling.

### Empty and error states

The interface should explain when:

- The dataset is still validating.
- No channels are available.
- The selected interval is invalid.
- The requested interval exceeds the allowed limit.
- The recording cannot be retrieved.
- The server returns no signal data.

### Completion criteria

An authorised user can select a ready dataset, request a bounded interval, and inspect its channels without loading the complete recording into the browser.

