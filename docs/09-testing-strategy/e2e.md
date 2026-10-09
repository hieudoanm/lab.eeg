## 15. End-to-End Tests

Use Playwright for a small number of critical user workflows.

### Workflow A: Create and analyse a dataset

1. Sign in using the supported test authentication configuration.
2. Select an authorised organisation.
3. Create a study.
4. Register a synthetic EEG dataset.
5. Upload the file.
6. Finalise the upload.
7. Wait for validation to complete.
8. Open the dataset detail page.
9. Submit a descriptive analysis.
10. Wait for the job to finish.
11. Open the result view.
12. Verify expected numerical output and provenance.

Use deterministic fixture data so the expected results are known.

### Workflow B: Permission boundaries

1. Sign in as a viewer.
2. Open a study and dataset.
3. Verify that read access works.
4. Verify that write controls are unavailable or disabled appropriately.
5. Attempt the corresponding protected API operation.
6. Verify that the backend rejects it.

Hiding a button is not sufficient proof of authorisation.

### Workflow C: Failure handling

1. Attempt to upload an invalid recording.
2. Verify that validation fails.
3. Verify that the dataset never becomes ready.
4. Verify that the UI explains the failure safely.
5. Verify that no successful analysis can be submitted for the invalid dataset.

