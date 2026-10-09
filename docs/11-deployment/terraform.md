## 7. Terraform Infrastructure

Terraform should define the infrastructure required by the selected production environment.

### Suggested structure

```text
infrastructure/
└── terraform/
    ├── modules/
    │   ├── api/
    │   ├── database/
    │   ├── storage/
    │   ├── worker/
    │   └── networking/
    └── environments/
        ├── dev/
        └── prod/
```

Keep modules small and introduce them only where reuse or separation of concerns is valuable.

### Infrastructure responsibilities

Terraform should manage, where supported:

- Cloud Run services.
- Cloud SQL configuration.
- Storage buckets.
- IAM bindings.
- Networking.
- Secret resource definitions and access policies.
- Logging and monitoring configuration.
- Budget alerts where available.
- Required APIs and service identities.

Secret values should not be committed to Terraform files or checked-in variable files.

Avoid placing secret values in Terraform state. Marking a variable as sensitive does not prevent it from being stored in state.

### Terraform state

Use a remote state backend with access controls, versioning or recovery support where available, and a documented locking strategy.

Requirements:

- Keep state out of Git.
- Restrict who can read and modify state.
- Separate environments.
- Review plans before applying changes.
- Avoid concurrent conflicting applies.
- Document state recovery.

### Infrastructure acceptance criteria

- Terraform validates.
- Plans are reviewed.
- IAM follows least privilege.
- Public exposure is intentional.
- Storage is private by default.
- Database networking is restricted.
- State is protected.
- Teardown instructions are tested for disposable environments.

