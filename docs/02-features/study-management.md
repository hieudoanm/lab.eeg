## 5. Dataset Upload and Validation

**Priority:** P0

### Description

Researchers upload EEG recordings and metadata into a study. EEG Lab validates the recording before making it available for exploration or analysis.

### Supported initial format

The initial implementation should choose and document one practical format:

- NumPy `.npy` recording plus a JSON metadata manifest; or
- EDF, using a maintained EEG-compatible parser.

Do not claim support for a format until its parser, validation, and tests are implemented.

### Upload workflow

<list>
  <list-item>**Select a study.** The user opens a study and starts a new dataset upload.</list-item>
  <list-item>**Enter metadata.** The user provides a dataset name, source information, sampling frequency, channel names, units, and other required metadata.</list-item>
  <list-item>**Select the recording.** The interface checks basic file constraints before uploading.</list-item>
  <list-item>**Upload the file.** The file is transferred to private storage, directly or through a controlled backend endpoint.</list-item>
  <list-item>**Validate the recording.** The backend verifies the file, parses the recording, and checks consistency with the manifest.</list-item>
  <list-item>**Display the result.** The interface shows whether validation succeeded, failed, or is still in progress.</list-item>
</list>

### Validation behaviour

Validation should check:

- Supported format.
- File size.
- File integrity and checksum.
- Expected dimensions.
- Numeric data validity.
- Sampling frequency.
- Channel metadata.
- Resource limits.
- Compatibility between the recording and its manifest.

Validation errors must identify the problem without revealing internal paths, stack traces, or implementation details.

### Dataset lifecycle

`pending_upload` → `validating` → `ready`

Alternative outcomes:

- `rejected`: input violates the supported format or validation rules.
- `failed`: an operational error prevents validation from completing.

A failed validation must not be treated as a successful upload.

### Dataset details

The dataset page should display:

- Name and recording identifier.
- Source and provenance.
- File format and checksum.
- Sampling frequency.
- Channel count and names.
- Number of samples and recording duration.
- Units, where known.
- Validation status.
- Upload and validation timestamps.
- Available visualisation and analysis actions.

### Edge cases

- Interrupted upload.
- Duplicate upload request.
- Missing metadata.
- Unsupported file.
- Malformed manifest.
- Checksum mismatch.
- File content does not match its extension.
- Non-finite signal values.
- Extremely long recording.
- Too many channels.
- Storage service unavailable.
- User loses permission during upload.
- Uploaded object exists but database finalisation fails.

### Important design rule

Never load an unbounded recording entirely into API memory merely to validate it. Apply file-size and resource limits, and use chunked or bounded processing where appropriate.

