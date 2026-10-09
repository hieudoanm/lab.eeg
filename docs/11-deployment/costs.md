## 14. Cost Management

A portfolio project should not incur unnecessary ongoing cloud costs.

Before deployment:

- Estimate recurring database, storage, compute, networking, logging, and broker costs.
- Prefer the smallest suitable resource configuration.
- Set budget notifications where supported.
- Configure service scaling limits.
- Limit worker concurrency.
- Set upload size and retention limits.
- Avoid keeping unused environments running.
- Document which services continue to incur charges while idle.

A budget alert is not necessarily a spending cap.

Review current regional pricing and free-tier eligibility before provisioning resources. Do not assume that a service remains free simply because a demonstration is small.

