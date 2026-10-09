## 8. Scientific Computing and EEG Data

### 8.1 NumPy

Use NumPy for numerical operations and descriptive statistics.

The implementation should document:

- Expected array dimensions.
- Channel ordering.
- Sampling-frequency units.
- Signal measurement units.
- Standard deviation convention.
- Handling of invalid values.
- Numerical precision expectations.

Avoid copying large arrays unnecessarily.

### 8.2 Initial recording format

For the simplest MVP, use a documented format based on:

- A NumPy `.npy` array.
- A JSON manifest containing the sampling frequency, channel names, units, and relevant provenance.

The manifest must be validated against the actual recording.

The implementation must explicitly document array orientation, for example `(channels, samples)`, and enforce it consistently.

Do not use Python pickle files for untrusted uploads.

If EDF support becomes a core requirement, introduce a maintained EEG-compatible parser as a separate, tested feature.

### 8.3 Resource limits

Set configurable limits for:

- Maximum upload size.
- Maximum channel count.
- Maximum recording duration.
- Maximum sample count.
- Maximum analysis execution time.
- Maximum visualisation interval.
- Maximum API response size.

Reject inputs that exceed safe limits before performing expensive processing.

