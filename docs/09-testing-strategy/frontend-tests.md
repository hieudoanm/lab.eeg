## 13. Frontend Component Tests

Use Vitest and React Testing Library to test user-visible behaviour.

Test components such as:

- Study creation form.
- Dataset upload form.
- Upload progress and validation states.
- Dataset metadata panel.
- Channel selector.
- EEG chart container.
- Analysis submission control.
- Job status indicator.
- Result summary.
- Error messages.
- Empty states.

Verify:

- Inputs have accessible labels.
- Keyboard navigation works for interactive controls.
- Loading states are displayed.
- Validation errors are understandable.
- Failed API requests do not display fabricated success.
- Viewers do not see enabled write controls.
- Data refreshes correctly after mutations.
- Long-running jobs show current status.

Do not test implementation details such as internal component state when the same behaviour can be tested through the rendered interface.

