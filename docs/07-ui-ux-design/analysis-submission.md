## 13. Analysis Submission and Job Detail

### Analysis submission

Use a simple form or dialog showing:

- Dataset name.
- Analysis type.
- Parameters supported by the selected analysis.
- A concise description of the computation.
- Submit and cancel actions.

Do not expose parameters that the backend does not support.

### Job detail layout

Display:

- Job identifier.
- Status.
- Analysis type.
- Dataset.
- Submitted timestamp.
- Started and completed timestamps when available.
- Algorithm version.
- Parameters.
- Safe error information or a link to the result.

### Status presentation

Use a text label and consistent visual indicator for each state:

| State     | Presentation                         |
| --------- | ------------------------------------ |
| Queued    | Neutral label and queued text        |
| Running   | Progress indicator and running text  |
| Succeeded | Success indicator and succeeded text |
| Failed    | Error indicator and failed text      |
| Cancelled | Neutral label and cancelled text     |

A running indicator must not imply a known percentage of completion unless the backend actually provides meaningful progress information.

