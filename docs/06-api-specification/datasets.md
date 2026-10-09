## 9. Dataset Endpoints

### `POST /organisations/{organisation_id}/studies/{study_id}/datasets`

Registers a dataset before upload.

Request:

```json
{
  "name": "Participant Demo 001",
  "description": "Synthetic EEG recording for application testing.",
  "format": "npy",
  "source_description": "Generated synthetic data",
  "metadata": { "signal_type": "eeg" }
}
```

Response: `201 Created`

```json
{
  "id": "2b4c6d8e-1111-4222-8333-abcdefabcdef",
  "study_id": "8f5d9c11-9b8d-4b65-9e10-4a83f7c0a112",
  "name": "Participant Demo 001",
  "format": "npy",
  "status": "pending_upload",
  "created_at": "2026-10-09T16:30:00Z"
}
```

The backend generates the dataset identifier and establishes organisation ownership. The client cannot choose an arbitrary storage key or mark a dataset as ready.

### `GET /organisations/{organisation_id}/datasets/{dataset_id}`

Returns dataset metadata and lifecycle state.

Example:

```json
{
  "id": "2b4c6d8e-1111-4222-8333-abcdefabcdef",
  "study_id": "8f5d9c11-9b8d-4b65-9e10-4a83f7c0a112",
  "name": "Participant Demo 001",
  "format": "npy",
  "status": "ready",
  "sampling_frequency_hz": 256.0,
  "channel_count": 8,
  "sample_count": 76800,
  "duration_seconds": 300.0,
  "channel_names": ["F3", "F4", "C3", "C4", "P3", "P4", "O1", "O2"],
  "checksum_sha256": "example_checksum_value",
  "validated_at": "2026-10-09T16:35:00Z"
}
```

The checksum above is illustrative, not a valid checksum to reuse.

Do not expose private object-storage keys or signed URLs through ordinary dataset responses.

### `GET /organisations/{organisation_id}/datasets/{dataset_id}/signals`

Returns a bounded, decimated window of a ready dataset's stored recording for charting. The read is display-only: it never feeds analysis and never mutates the stored recording.

Query parameters:

| Name            | Required | Bounds                          | Description                                              |
| --------------- | -------- | ------------------------------- | -------------------------------------------------------- |
| `start_seconds` | No       | `>= 0`, default `0`             | Start of the window, in seconds from the recording start |
| `end_seconds`   | Yes      | `> 0`                           | End of the window, in seconds; must be after `start`     |
| `channels`      | No       | comma-separated channel indices | Selected channels; defaults to the first channel (`0`)   |

Response: `200 OK`

```json
{
  "dataset_id": "2b4c6d8e-1111-4222-8333-abcdefabcdef",
  "sampling_frequency_hz": 256.0,
  "channel_count": 8,
  "sample_count": 76800,
  "duration_seconds": 300.0,
  "start_seconds": 1.0,
  "end_seconds": 2.0,
  "sample_start": 256,
  "sample_end": 512,
  "channel_indices": [0, 1],
  "points_per_channel": 256,
  "downsampled": false,
  "downsample_factor": 1,
  "time_seconds": [1.0, 1.00390625, 1.0078125],
  "channels": [
    {
      "channel_index": 0,
      "channel_name": "F3",
      "unit": "uV",
      "samples": [0.02, -0.11, 0.34]
    }
  ]
}
```

Validation and errors:

- The dataset must belong to the requested organisation; a foreign dataset is `404 Not Found`.
- The dataset must be `ready`; otherwise `409 Conflict` with code `dataset_not_ready`.
- A ready dataset whose recording object is missing returns `409 Conflict` with code `dataset_recording_missing`.
- A recording that cannot be parsed returns `409 Conflict` with code `dataset_recording_invalid`.
- An invalid window (for example `end_seconds <= start_seconds`, a window beyond the recording, or a window longer than the configured maximum) returns `422 Unprocessable Entity` with code `invalid_signal_window`.
- Non-integer channel values return `422 Unprocessable Entity` with code `invalid_signal_window`.
- A channel selection that resolves to no valid channels returns `422 Unprocessable Entity` with code `invalid_channel_selection`.

The response is bounded: the service decimates so that no more than the configured maximum number of points per channel is returned, setting `downsampled` and `downsample_factor` accordingly. Non-finite samples are reported as `null` so charts render a gap, and `samples` always align with the shared `time_seconds` axis. This endpoint feeds display only and must never be used as input to analysis.

