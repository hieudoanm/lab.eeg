## 14. Visualisation Tests

Scientific charts require additional correctness checks.

Test that:

- The time axis uses the supplied sampling frequency.
- Channel selection displays the correct channel.
- Displayed statistics match API results.
- Unknown units are not presented as known units.
- Empty and invalid data have meaningful states.
- Large recordings respect documented rendering limits.
- Downsampling, when implemented, does not alter persisted analysis results.
- Chart labels and alternative summaries are available.

A chart that renders without throwing an exception is not necessarily scientifically correct.

