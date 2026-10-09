## 11. Infrastructure as Code

Use Terraform for cloud infrastructure.

Manage infrastructure resources such as:

- Application service configuration.
- Database instances.
- Object-storage buckets.
- Service accounts and IAM bindings.
- Secrets references.
- Logging and monitoring configuration.
- Networking and access controls.
- Required environment configuration.

Requirements:

- Do not commit Terraform state files or secrets.
- Use remote state for shared deployments.
- Enable state locking where supported.
- Review plans before applying changes.
- Separate environments where practical.
- Avoid granting broad project-level permissions when narrower permissions suffice.
- Document resource naming and ownership.

For the MVP, Terraform can be introduced after the application works locally, but infrastructure should be reproducible before claiming a production deployment.

