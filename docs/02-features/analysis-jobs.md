## 8. Job Monitoring and Analysis History

**Priority:** P1

### Description

Researchers need to inspect recent and historical analyses for a study or dataset.

### Behaviour

The interface should support:

- Listing jobs.
- Filtering by status.
- Viewing job details.
- Opening successful results.
- Inspecting safe error details for failed jobs.
- Retrying eligible failures.
- Distinguishing a new run from a previous result.

### Retry semantics

A retry should be an explicit, auditable operation.

The system must distinguish:

- Retrying an existing job, where supported.
- Creating a new job from the same dataset and parameters.
- Creating a job against a newer dataset version.

Historical results must remain attributable to the original inputs and algorithm version.

