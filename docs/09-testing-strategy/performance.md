## 17. Performance and Resource Tests

Performance testing should focus on the application's likely bottlenecks.

Test:

- Dataset-list queries at representative sizes.
- Pagination behaviour.
- Concurrent analysis submissions.
- Worker processing time for supported dataset sizes.
- Memory use while loading recordings.
- Chart rendering with maximum supported sample counts.
- Database connection-pool behaviour.
- Recovery after a temporary dependency outage.

Define realistic limits for the MVP before claiming support for large recordings.

Do not run expensive benchmarks on every pull request. Use targeted smoke tests in CI and scheduled or release-oriented load tests where justified.

