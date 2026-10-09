## 5. HTTP Status Codes

| Status                       | Meaning                                       |
| ---------------------------- | --------------------------------------------- |
| `200 OK`                     | Successful read or update                     |
| `201 Created`                | Resource created                              |
| `202 Accepted`               | Asynchronous operation accepted               |
| `204 No Content`             | Successful operation with no response body    |
| `400 Bad Request`            | Malformed or invalid request                  |
| `401 Unauthorized`           | Missing or invalid authentication             |
| `403 Forbidden`              | Authenticated caller lacks permission         |
| `404 Not Found`              | Resource missing or intentionally concealed   |
| `409 Conflict`               | State conflict or idempotency-key mismatch    |
| `413 Content Too Large`      | Upload exceeds the configured limit           |
| `415 Unsupported Media Type` | Unsupported upload format                     |
| `422 Unprocessable Entity`   | Request schema or semantic validation failure |
| `429 Too Many Requests`      | Rate limit exceeded                           |
| `500 Internal Server Error`  | Unexpected server failure                     |
| `503 Service Unavailable`    | Required service temporarily unavailable      |

Use `409 Conflict` for invalid state transitions and conflicting reuse of an idempotency key.

