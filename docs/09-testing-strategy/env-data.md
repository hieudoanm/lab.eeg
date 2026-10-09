## 20. Test Data and Environment Management

Use synthetic test data exclusively for ordinary development and CI.

Requirements:

- Keep fixtures small and deterministic.
- Generate larger files only for resource-limit tests.
- Avoid committing large binary recordings unnecessarily.
- Keep secrets out of fixture files.
- Use isolated test databases.
- Clean up temporary storage objects.
- Avoid shared mutable fixtures across tests.
- Ensure parallel tests cannot interfere with one another.

If real research data is ever introduced, establish an explicit approval, access, retention, and anonymisation process first.

## 21. Coverage and Quality Targets

Code coverage is a diagnostic, not the primary success criterion. Do not set a
single line-coverage percentage as a release gate.

Prioritise instead:

- High coverage of numerical edge cases.
- Complete coverage of critical role and tenant-isolation scenarios.
- Explicit tests for all supported analysis state transitions.
- End-to-end coverage of the primary workflow.

Do not chase coverage by testing trivial getters or writing assertions that provide little confidence.

Critical security boundaries and scientific algorithms deserve stronger targeted tests than their raw line coverage might suggest.

## 22. Defect Handling

When a defect is found:

1. Reproduce the failure.
2. Determine the affected boundary.
3. Add a regression test.
4. Fix the underlying cause.
5. Run the relevant test suite.
6. Review whether other endpoints or workflows share the same defect.
7. Document the impact if it affects scientific results, security, or persisted data.

For defects that could have altered persisted scientific results, consider whether existing results need to be invalidated, recomputed, or marked with an affected algorithm version.

## 23. Definition of Done

A feature is complete when:

1. Its expected behaviour is tested.
2. Invalid inputs are handled.
3. Authorisation is enforced server-side.
4. Relevant database constraints are tested.
5. Scientific calculations have known expected outputs where applicable.
6. Failure and retry behaviour are covered for asynchronous work.
7. Frontend loading, success, empty, and error states are implemented.
8. The relevant automated checks pass.
9. No sensitive test data or credentials are committed.
10. Documentation and API contracts are updated.
11. The feature does not introduce unsupported medical or diagnostic claims.

Place frontend tests in colocated `__tests__` directories next to the code they exercise.
