## Recovery Time Objectives (RTO) / Recovery Point Objectives (RPO)

| Component                            | RTO       | RPO       | Notes                     |
| ------------------------------------ | --------- | --------- | ------------------------- |
| PostgreSQL (managed, PITR)           | < 30 min  | < 5 min   | Automated failover        |
| PostgreSQL (self-hosted, daily dump) | < 2 hours | 24 hours  | Manual restore            |
| Object Storage (GCS, versioned)      | < 15 min  | Near-zero | Versioning                |
| Object Storage (local, rsync)        | < 1 hour  | 24 hours  | Depends on sync frequency |

