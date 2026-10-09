## 13. Cancellation

### `POST /organisations/{organisation_id}/analyses/{analysis_id}/cancel`

Requests cancellation of an analysis.

Cancellation is cooperative: a worker may already be executing when the request arrives.

Rules:

- A queued job may be cancelled before execution.
- A running job may receive a cancellation request if supported.
- A succeeded job cannot be cancelled retroactively.
- Repeated cancellation requests must be safe.
- State transitions must be persisted.
- Workers must check the current state before committing results.

Return a response that reflects the actual persisted state. Do not report cancellation as complete while the job is still running.

If the initial implementation cannot reliably interrupt running computations, document that limitation and support cancellation only before execution begins.

