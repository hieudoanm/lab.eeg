# EEG Lab Infrastructure — Terraform Modules

This directory contains reusable Terraform modules for deploying EEG Lab to GCP.

## Module Structure

```
modules/
├── network/           # VPC, subnets, Cloud NAT, firewall rules
├── cloudsql/          # PostgreSQL instance, database, users
├── memorystore/       # Redis instance
├── storage/           # GCS buckets (recordings, artifacts)
├── secrets/           # Secret Manager secrets
├── service-accounts/  # IAM service accounts with least-privilege roles
├── cloudrun/          # Cloud Run services (API, worker, frontend, dispatcher)
├── scheduler/         # Cloud Scheduler jobs (dispatcher cron)
└── monitoring/        # Log-based metrics, alerting policies, dashboards
```

## Environments

```
environments/
├── dev/               # Development environment (minimal resources)
└── prod/              # Production environment (HA, larger instances)
```

Each environment has its own `main.tf`, `variables.tf`, and `terraform.tfvars`.

## Usage

```bash
# Initialize
cd infrastructure/terraform/environments/dev
terraform init

# Plan
terraform plan -var="project_id=my-gcp-project" -var="region=us-central1"

# Apply
terraform apply -var="project_id=my-gcp-project" -var="region=us-central1"
```

## Required APIs

Enable these APIs in your GCP project before applying:

```bash
gcloud services enable \
  run.googleapis.com \
  sqladmin.googleapis.com \
  redis.googleapis.com \
  storage.googleapis.com \
  secretmanager.googleapis.com \
  cloudbuild.googleapis.com \
  artifactregistry.googleapis.com \
  logging.googleapis.com \
  monitoring.googleapis.com \
  scheduler.googleapis.com \
  vpcaccess.googleapis.com \
  servicenetworking.googleapis.com \
  iam.googleapis.com \
  cloudresourcemanager.googleapis.com
```

## State Backend

Configure remote state in each environment's `backend.tf`:

```hcl
terraform {
  backend "gcs" {
    bucket  = "eeg-lab-terraform-state-${var.project_id}"
    prefix  = "environments/dev"
  }
}
```

Create the state bucket:

```bash
gsutil mb -p ${PROJECT_ID} -l us-central1 gs://eeg-lab-terraform-state-${PROJECT_ID}
gsutil versioning set on gs://eeg-lab-terraform-state-${PROJECT_ID}
```

## Naming Convention

All resources follow: `eeg-lab-${environment}-${resource-type}-${suffix}`

Examples:

- `eeg-lab-dev-api`
- `eeg-lab-prod-cloudsql`
- `eeg-lab-dev-redis`

## Security Principles

1. **Least privilege** — Service accounts get only required roles
2. **Private networking** — Cloud SQL and Redis use private IPs; VPC connector for Cloud Run
3. **No public IPs** — Cloud Run services are internal + Cloud Run ingress control
4. **Secrets in Secret Manager** — No plaintext secrets in Terraform or CI
5. **Workload Identity Federation** — GitHub Actions authenticates via OIDC, no service account keys
6. **Encryption** — CMEK for Cloud SQL, Redis, Storage (optional, using KMS)

## Cost Controls

- Dev: `min_instances = 0` for workers, `db-f1-micro` Cloud SQL, Basic Redis
- Prod: HA Cloud SQL, Standard Redis HA, min instances for availability
- All: CPU throttling on Cloud Run (only pay during requests)
