## 6. Scientific Correctness Tests

Scientific calculations need explicit mathematical expectations. Tests must not merely check that a result exists or that a function returns a number.

### Initial analysis

For each EEG channel, EEG Lab calculates:

- Mean.
- Standard deviation.
- Minimum.
- Maximum.
- Peak-to-peak range.
- Valid sample count.

Define the treatment of non-finite values and the standard-deviation convention in the algorithm specification.

For the initial implementation, a reasonable contract is:

- Exclude non-finite values from descriptive statistics.
- Report the valid sample count.
- Use population standard deviation (`ddof=0`).
- Reject a channel with no valid samples.
- Reject an entire recording if its structure or metadata is invalid.

If the implementation adopts different behaviour, document it explicitly and update the tests.

### Example fixture

```python
import numpy as np

signal = np.array([1.0, 2.0, 3.0, 4.0])
```

Expected values:

| Metric                        |         Expected value |
| ----------------------------- | ---------------------: |
| Mean                          |                    2.5 |
| Population standard deviation | Approximately 1.118034 |
| Minimum                       |                    1.0 |
| Maximum                       |                    4.0 |
| Peak-to-peak range            |                    3.0 |
| Valid sample count            |                      4 |

Use floating-point tolerances for comparisons rather than requiring exact binary equality.

Example:

```python
assert result["mean"] == pytest.approx(2.5)
assert result["standard_deviation"] == pytest.approx(
    np.std(signal, ddof=0)
)
```

### Required numerical test cases

- Constant-valued signals.
- Positive and negative values.
- Single-sample channels.
- Missing or non-finite samples.
- All-invalid channels.
- Very small and large valid values.
- Empty arrays.
- Incorrect dimensions.
- Unsupported dtypes.
- Channel metadata that does not match array dimensions.
- Invalid sampling frequency.

The accepted behaviour must be consistent between the numerical implementation, API contract, and UI.

### Scientific invariants

For every successfully analysed channel:

- Mean is finite.
- Standard deviation is finite and non-negative.
- Minimum does not exceed maximum.
- Peak-to-peak range is non-negative.
- Valid sample count is positive.
- Peak-to-peak range equals maximum minus minimum within the defined numerical tolerance.

These invariants are useful checks, but they do not replace comparison against known expected results.

