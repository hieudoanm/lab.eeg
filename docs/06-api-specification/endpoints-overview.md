## 6. Endpoint Overview

All organisation-scoped routes begin with:

`/organisations/{organisation_id}`

| Method   | Endpoint                                                                                       | Purpose                                      |
| -------- | ---------------------------------------------------------------------------------------------- | -------------------------------------------- |
| `GET`    | `/me`                                                                                          | Return the current user and memberships      |
| `POST`   | `/organisations`                                                                               | Create an organisation and make caller owner |
| `GET`    | `/organisations`                                                                               | List organisations accessible to the caller  |
| `GET`    | `/organisations/{organisation_id}`                                                             | Get organisation details                     |
| `GET`    | `/organisations/{organisation_id}/summary`                                                     | Get organisation dashboard summary           |
| `GET`    | `/organisations/{organisation_id}/memberships`                                                 | List organisation members                    |
| `POST`   | `/organisations/{organisation_id}/memberships`                                                 | Add a member                                 |
| `PATCH`  | `/organisations/{organisation_id}/memberships/{membership_id}`                                 | Change a member's role                       |
| `DELETE` | `/organisations/{organisation_id}/memberships/{membership_id}`                                 | Remove a member                              |
| `GET`    | `/organisations/{organisation_id}/studies`                                                     | List studies                                 |
| `POST`   | `/organisations/{organisation_id}/studies`                                                     | Create a study                               |
| `GET`    | `/organisations/{organisation_id}/studies/{study_id}`                                          | Get a study                                  |
| `PATCH`  | `/organisations/{organisation_id}/studies/{study_id}`                                          | Update a study                               |
| `POST`   | `/organisations/{organisation_id}/studies/{study_id}/archive`                                  | Archive a study                              |
| `GET`    | `/organisations/{organisation_id}/studies/{study_id}/datasets`                                 | List study datasets                          |
| `POST`   | `/organisations/{organisation_id}/studies/{study_id}/datasets`                                 | Register dataset metadata                    |
| `GET`    | `/organisations/{organisation_id}/datasets/{dataset_id}`                                       | Get dataset metadata                         |
| `GET`    | `/organisations/{organisation_id}/datasets/{dataset_id}/signals`                               | Retrieve a bounded signal window             |
| `POST`   | `/organisations/{organisation_id}/datasets/{dataset_id}/upload-sessions`                       | Create an upload session                     |
| `PUT`    | `/organisations/{organisation_id}/datasets/{dataset_id}/upload-sessions/{session_id}/content`  | Upload recording content (backend-mediated)  |
| `POST`   | `/organisations/{organisation_id}/datasets/{dataset_id}/upload-sessions/{session_id}/finalise` | Finalise and validate an upload              |
| `GET`    | `/organisations/{organisation_id}/datasets/{dataset_id}/analyses`                              | List dataset analyses                        |
| `POST`   | `/organisations/{organisation_id}/datasets/{dataset_id}/analyses`                              | Submit an analysis                           |
| `GET`    | `/organisations/{organisation_id}/analyses`                                                    | List organisation analyses                   |
| `GET`    | `/organisations/{organisation_id}/analyses/{analysis_id}`                                      | Get analysis status                          |
| `GET`    | `/organisations/{organisation_id}/analyses/{analysis_id}/results`                              | Retrieve successful analysis results         |
| `POST`   | `/organisations/{organisation_id}/analyses/{analysis_id}/cancel`                               | Request cancellation                         |

Implement only endpoints required by the accepted MVP features. Do not create placeholder endpoints that return fake success responses.

The implementation also exposes unauthenticated liveness and readiness probes at `/health/live` and `/health/ready` (outside the `/api/v1` prefix). `/health/ready` returns `503` with its readiness representation when the database is unavailable.

