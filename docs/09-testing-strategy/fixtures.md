## 7. Synthetic EEG Fixtures

Create deterministic synthetic datasets for testing.

Recommended fixtures:

1. A simple fixed numerical array for exact statistics.
2. A multichannel signal with different known channel values.
3. A sinusoidal signal for time-axis and visualisation tests.
4. A signal containing non-finite samples.
5. An empty array.
6. An array with the wrong orientation.
7. A malformed or unsupported file.
8. An array that exceeds configured resource limits.

For repeatable generated signals, use a fixed random seed.

Never make tests depend on random values generated afresh without a deterministic seed or an explicit property-based testing strategy.

Do not use real participant recordings merely because they are convenient test fixtures.

