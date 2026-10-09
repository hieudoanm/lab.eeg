# Infrastructure Costs and Teardown Instructions

This document provides cost estimates for deploying EEG Lab to Google Cloud Platform and instructions for tearing down resources.

## Estimated Monthly Costs (GCP, us-central1)

> **Disclaimer**: Estimates based on GCP pricing (2026). Actual costs vary by region, usage, and discounts. Use the [GCP Pricing Calculator](https://cloud.google.com/products/calculator) for precise estimates.

### Baseline Configuration (Development/Staging)

| Service                    | Configuration                                                | Est. Monthly Cost |
| -------------------------- | ------------------------------------------------------------ | ----------------- |
| **Cloud Run (API)**        | 2 vCPU, 1 GiB, 3 instances (min 1), 1M requests              | ~$15              |
| **Cloud Run (Worker)**     | 2 vCPU, 2 GiB, 2 instances (min 0), CPU only during requests | ~$8               |
| **Cloud Run (Frontend)**   | 1 vCPU, 512 MiB, 2 instances (min 1), 500k requests          | ~$5               |
| **Cloud Run (Dispatcher)** | 1 vCPU, 512 MiB, 1 instance (min 0), scheduled every 30s     | ~$2               |
| **Cloud SQL (PostgreSQL)** | db-f1-micro (1 vCPU, 0.6 GiB), 10 GB SSD, 7-day backups      | ~$15              |
| **Memorystore (Redis)**    | Basic tier, 1 GB, us-central1                                | ~$35              |
| **Cloud Storage**          | 10 GB Standard, 1M Class A ops, 10M Class B ops              | ~$0.30            |
| **Secret Manager**         | 10 secrets, 10k accesses                                     | ~$0.50            |
| **Cloud Logging**          | 50 GB ingested, 30-day retention                             | ~$25              |
| **Cloud Monitoring**       | 100 metrics, 1M API calls                                    | ~$5               |
| **Cloud Build**            | 120 build-minutes/day                                        | ~$4               |
| **Artifact Registry**      | 10 GB Docker images                                          | ~$1               |
| **VPC / Networking**       | Cloud NAT, Load Balancer (if used)                           | ~$10              |
| **Total (Baseline)**       |                                                              | **~$126/month**   |

### Production Configuration (Higher Availability)

| Service                    | Configuration                                                      | Est. Monthly Cost |
| -------------------------- | ------------------------------------------------------------------ | ----------------- |
| **Cloud Run (API)**        | 2 vCPU, 2 GiB, 5 instances (min 3), 10M requests                   | ~$120             |
| **Cloud Run (Worker)**     | 4 vCPU, 4 GiB, 5 instances (min 2)                                 | ~$80              |
| **Cloud Run (Frontend)**   | 2 vCPU, 1 GiB, 3 instances (min 2)                                 | ~$25              |
| **Cloud SQL (PostgreSQL)** | db-custom-4-16384 (4 vCPU, 16 GiB), 100 GB SSD, HA, 30-day backups | ~$350             |
| **Memorystore (Redis)**    | Standard tier HA, 5 GB, cross-zone                                 | ~$200             |
| **Cloud Storage**          | 500 GB Standard, versioning, 10M ops                               | ~$12              |
| **Cloud Logging**          | 200 GB ingested, 90-day retention                                  | ~$100             |
| **Cloud Monitoring**       | 500 metrics, 10M API calls, uptime checks                          | ~$25              |
| **Cloud Armor (WAF)**      | 1M requests, 10 rules                                              | ~$15              |
| **Total (Production)**     |                                                                    | **~$927/month**   |

### Cost Optimisation Tips

1. **Use min-instances=0** for workers and dispatcher (scales to zero when idle)
2. **Enable Cloud SQL autoscaling** (storage only; compute requires instance restart)
3. **Set Memorystore to Basic tier** for dev/staging (no HA)
4. **Reduce logging verbosity** in production (`log_level=WARNING`)
5. **Use Cloud Run CPU throttling** (only allocate CPU during requests)
6. **Lifecycle rules on Storage** — delete old versions after 90 days
7. **Committed Use Discounts** for Cloud SQL/Memorystore (1yr/3yr)

## Teardown Instructions

### Quick Teardown (Destroy All Resources)

```bash
# 1. Set project
export PROJECT_ID="your-gcp-project-id"
gcloud config set project $PROJECT_ID

# 2. Delete Cloud Run services
gcloud run services delete eeg-lab-api --region=us-central1 --quiet
gcloud run services delete eeg-lab-worker --region=us-central1 --quiet
gcloud run services delete eeg-lab-frontend --region=us-central1 --quiet
gcloud run services delete eeg-lab-dispatcher --region=us-central1 --quiet

# 3. Delete Cloud Scheduler job
gcloud scheduler jobs delete eeg-lab-dispatcher --location=us-central1 --quiet

# 4. Delete Cloud SQL instance (includes backups!)
gcloud sql instances delete eeg-lab-db --quiet

# 5. Delete Memorystore instance
gcloud redis instances delete eeg-lab-redis --region=us-central1 --quiet

# 6. Delete Cloud Storage bucket (WARNING: deletes all recordings!)
gsutil -m rm -r gs://eeg-lab-uploads-${PROJECT_ID}

# 7. Delete Secret Manager secrets
gcloud secrets delete eeg-lab-auth-token-secret --quiet
gcloud secrets delete eeg-lab-db-password --quiet
gcloud secrets delete eeg-lab-redis-auth --quiet

# 8. Delete Service Accounts
gcloud iam service-accounts delete eeg-lab-api@${PROJECT_ID}.iam.gserviceaccount.com --quiet
gcloud iam service-accounts delete eeg-lab-worker@${PROJECT_ID}.iam.gserviceaccount.com --quiet
gcloud iam service-accounts delete eeg-lab-frontend@${PROJECT_ID}.iam.gserviceaccount.com --quiet

# 9. Disable APIs (optional, to prevent accidental charges)
gcloud services disable run.googleapis.com sqladmin.googleapis.com redis.googleapis.com \
  storage.googleapis.com secretmanager.googleapis.com cloudbuild.googleapis.com \
  artifactregistry.googleapis.com logging.googleapis.com monitoring.googleapis.com \
  scheduler.googleapis.com --quiet
```

### Selective Teardown (Keep Data)

```bash
# Keep Cloud SQL and Storage, only delete compute
gcloud run services delete eeg-lab-api eeg-lab-worker eeg-lab-frontend eeg-lab-dispatcher \
  --region=us-central1 --quiet

gcloud scheduler jobs delete eeg-lab-dispatcher --location=us-central1 --quiet

# Service accounts can remain for future redeploy
```

### Terraform Teardown (If Using IaC)

```bash
cd infrastructure/terraform

# Review what will be destroyed
terraform plan -destroy -var="project_id=${PROJECT_ID}" -var="region=us-central1"

# Destroy (requires confirmation)
terraform destroy -var="project_id=${PROJECT_ID}" -var="region=us-central1"

# Clean up state (optional)
rm -rf .terraform terraform.tfstate*
```

### Local Development Teardown

```bash
# Stop Docker containers
make infra-down
# or
docker compose -f docker-compose.yml down -v

# Remove local uploads
rm -rf .local-data/uploads

# Remove Python virtual environment
rm -rf packages/backend/.venv

# Remove Node modules
rm -rf packages/frontend/node_modules packages/frontend/.next

# Remove lock files (regenerated on next install)
rm packages/backend/uv.lock packages/frontend/pnpm-lock.yaml
```

## Budget Alerts (Recommended)

```bash
# Create budget alert at $50/month (dev) or $1000/month (prod)
gcloud billing budgets create \
  --billing-account=YOUR_BILLING_ACCOUNT_ID \
  --display-name="EEG Lab Dev Budget" \
  --budget-amount=50USD \
  --threshold-rule=percent=0.5 \
  --threshold-rule=percent=0.9 \
  --threshold-rule=percent=1.0 \
  --filter="projects=YOUR_PROJECT_ID"
```

## Cost Monitoring Dashboard

Create a custom Cloud Monitoring dashboard with:

1. **Cloud Run**: Request count, latency, instance count, CPU/memory utilisation
2. **Cloud SQL**: CPU, memory, disk usage, connections, backup status
3. **Memorystore**: Memory usage, hit rate, operations/sec
4. **Cloud Storage**: Storage bytes, object count, egress bandwidth
5. **Billing**: Daily cost, forecasted monthly cost

```bash
# Example: Export billing data to BigQuery for custom analysis
gcloud billing accounts list
bq mk --dataset PROJECT_ID:billing_export
gcloud billing projects link PROJECT_ID --billing-account=BILLING_ACCOUNT
# Then configure billing export to BigQuery in Console
```

---

_Review and update estimates quarterly. Actual costs depend on traffic patterns and GCP pricing changes._
