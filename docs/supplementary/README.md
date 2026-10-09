# Supplementary Documentation

This folder contains supporting documents that complement the 12 normative documents in `docs/`. These are not required for implementation but provide additional context, operational guidance, and portfolio artefacts.

| Document                                                     | Description                                                                                                                                   | Related Normative Doc                                   |
| ------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| [`architecture-diagrams.md`](architecture-diagrams.md)       | Mermaid diagrams: system architecture, request lifecycles, tenant isolation, outbox pattern, job state machine, data model, deployment target | `docs/05-architecture.md`, `docs/06-data-model.md`      |
| [`case-study.md`](case-study.md)                             | Portfolio case study: problem statement, engineering decisions, technical highlights, challenges, metrics                                     | `docs/01-overview.md`, `docs/11-implementation-plan.md` |
| [`database-backup-recovery.md`](database-backup-recovery.md) | Backup strategies (pg_dump, PITR), recovery procedures, RTO/RPO, monitoring alerts                                                            | `docs/12-deployment.md`, `docs/06-data-model.md`        |
| [`infrastructure-costs.md`](infrastructure-costs.md)         | GCP cost estimates (dev/prod), optimisation tips, teardown instructions, budget alerts                                                        | `docs/12-deployment.md`                                 |
| [`known-limitations.md`](known-limitations.md)               | Comprehensive limitations tracker across scientific, technical, operational, data model, documentation categories                             | `docs/11-implementation-plan.md`, `docs/decisions/`     |
| [`threat-model.md`](threat-model.md)                         | STRIDE threat model with trust boundaries, attack surface, controls matrix, residual risks                                                    | `docs/05-architecture.md`, `docs/12-deployment.md`      |

## Normative Documents (Core)

The following 12 documents in `docs/` are the authoritative source for implementation:

| #   | Document                    | Governs                                       |
| --- | --------------------------- | --------------------------------------------- |
| 01  | `01-overview.md`            | Product vision and scope                      |
| 02  | `02-requirements.md`        | Functional/non-functional requirements        |
| 03  | `03-user-stories.md`        | User goals and acceptance scenarios           |
| 04  | `04-features.md`            | User-visible behaviour                        |
| 05  | `05-architecture.md`        | Module boundaries, request lifecycle, tenancy |
| 06  | `06-data-model.md`          | Schema, constraints, state transitions        |
| 07  | `07-api-spec.md`            | REST contract, status codes, error envelope   |
| 08  | `08-tech-stack.md`          | Allowed technologies and dependency policy    |
| 09  | `09-design.md`              | UI tokens, layout, states, accessibility      |
| 10  | `10-testing-strategy.md`    | Test layers, fixtures, acceptance criteria    |
| 11  | `11-implementation-plan.md` | Phased delivery plan                          |
| 12  | `12-deployment.md`          | Operations and deployment                     |

When an implementation and a normative document disagree, the normative document wins.
