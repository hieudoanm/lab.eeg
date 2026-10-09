## 15. Definition of Feature Completion

A feature is complete only when:

1. Its intended behaviour is documented.
2. The frontend and backend implement the agreed contract.
3. Authorisation is enforced on the server.
4. Validation and failure states are handled.
5. Relevant automated tests pass.
6. Database migrations are included where required.
7. Logs and operational behaviour are adequate.
8. User-facing documentation is updated.
9. No critical security requirement is knowingly violated.
10. The feature has been exercised through an end-to-end workflow where practical.
11. Changed backend and frontend source files pass the repository's Husky pre-commit checks.
