# Service Accounts with Least-Privilege Roles

# API service account
resource "google_service_account" "api" {
  account_id   = "neurolab-${var.environment}-api"
  display_name = "NeuroLab ${var.environment} API"
  description  = "Service account for NeuroLab API (FastAPI)"
}

# Worker service account
resource "google_service_account" "worker" {
  account_id   = "neurolab-${var.environment}-worker"
  display_name = "NeuroLab ${var.environment} Worker"
  description  = "Service account for NeuroLab Celery Worker"
}

# Frontend service account
resource "google_service_account" "frontend" {
  account_id   = "neurolab-${var.environment}-frontend"
  display_name = "NeuroLab ${var.environment} Frontend"
  description  = "Service account for NeuroLab Frontend (Next.js)"
}

# Dispatcher service account
resource "google_service_account" "dispatcher" {
  account_id   = "neurolab-${var.environment}-dispatcher"
  display_name = "NeuroLab ${var.environment} Dispatcher"
  description  = "Service account for NeuroLab Outbox Dispatcher"
}

# Cloud Build service account
resource "google_service_account" "builder" {
  account_id   = "neurolab-${var.environment}-builder"
  display_name = "NeuroLab ${var.environment} Builder"
  description  = "Service account for Cloud Build"
}

# Migration service account (for applying SQL migrations)
resource "google_service_account" "migration" {
  account_id   = "neurolab-${var.environment}-migration"
  display_name = "NeuroLab ${var.environment} Migration"
  description  = "Service account for database migrations"
}

# IAM roles for API
resource "google_project_iam_member" "api_logging" {
  project = var.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.api.email}"
}

resource "google_project_iam_member" "api_monitoring" {
  project = var.project_id
  role    = "roles/monitoring.metricWriter"
  member  = "serviceAccount:${google_service_account.api.email}"
}

resource "google_project_iam_member" "api_trace" {
  project = var.project_id
  role    = "roles/cloudtrace.agent"
  member  = "serviceAccount:${google_service_account.api.email}"
}

# IAM roles for Worker
resource "google_project_iam_member" "worker_logging" {
  project = var.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.worker.email}"
}

resource "google_project_iam_member" "worker_monitoring" {
  project = var.project_id
  role    = "roles/monitoring.metricWriter"
  member  = "serviceAccount:${google_service_account.worker.email}"
}

resource "google_project_iam_member" "worker_trace" {
  project = var.project_id
  role    = "roles/cloudtrace.agent"
  member  = "serviceAccount:${google_service_account.worker.email}"
}

# IAM roles for Frontend
resource "google_project_iam_member" "frontend_logging" {
  project = var.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.frontend.email}"
}

# IAM roles for Dispatcher
resource "google_project_iam_member" "dispatcher_logging" {
  project = var.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.dispatcher.email}"
}

resource "google_project_iam_member" "dispatcher_monitoring" {
  project = var.project_id
  role    = "roles/monitoring.metricWriter"
  member  = "serviceAccount:${google_service_account.dispatcher.email}"
}

# IAM roles for Builder
resource "google_project_iam_member" "builder_storage_admin" {
  project = var.project_id
  role    = "roles/storage.admin"
  member  = "serviceAccount:${google_service_account.builder.email}"
}

resource "google_project_iam_member" "builder_artifactregistry" {
  project = var.project_id
  role    = "roles/artifactregistry.writer"
  member  = "serviceAccount:${google_service_account.builder.email}"
}

resource "google_project_iam_member" "builder_cloudbuild" {
  project = var.project_id
  role    = "roles/cloudbuild.builds.builder"
  member  = "serviceAccount:${google_service_account.builder.email}"
}

resource "google_project_iam_member" "builder_run_deployer" {
  project = var.project_id
  role    = "roles/run.deployer"
  member  = "serviceAccount:${google_service_account.builder.email}"
}

resource "google_project_iam_member" "builder_iam_sa_user" {
  project = var.project_id
  role    = "roles/iam.serviceAccountUser"
  member  = "serviceAccount:${google_service_account.builder.email}"
}

# IAM roles for Migration
resource "google_project_iam_member" "migration_sql_client" {
  project = var.project_id
  role    = "roles/cloudsql.client"
  member  = "serviceAccount:${google_service_account.migration.email}"
}

# Workload Identity Federation for GitHub Actions
resource "google_iam_workload_identity_pool" "github" {
  workload_identity_pool_id = "github-actions-${var.environment}"
  display_name              = "GitHub Actions ${var.environment}"
  description               = "Workload Identity Pool for GitHub Actions ${var.environment}"
}

resource "google_iam_workload_identity_pool_provider" "github" {
  workload_identity_pool_id          = google_iam_workload_identity_pool.github.workload_identity_pool_id
  workload_identity_pool_provider_id = "github-provider"
  display_name                       = "GitHub OIDC Provider"
  attribute_mapping = {
    "google.subject"       = "assertion.sub"
    "attribute.actor"      = "assertion.actor"
    "attribute.repository" = "assertion.repository"
  }
  attribute_condition = "assertion.repository_owner == '${var.github_org}'"
  oidc {
    issuer_uri = "https://token.actions.githubusercontent.com"
  }
}

# Allow GitHub Actions to impersonate service accounts
resource "google_service_account_iam_binding" "github_api" {
  service_account_id = google_service_account.api.name
  role               = "roles/iam.workloadIdentityUser"
  members            = ["principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github.name}/attribute.repository/${var.github_repo}"]
}

resource "google_service_account_iam_binding" "github_worker" {
  service_account_id = google_service_account.worker.name
  role               = "roles/iam.workloadIdentityUser"
  members            = ["principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github.name}/attribute.repository/${var.github_repo}"]
}

resource "google_service_account_iam_binding" "github_frontend" {
  service_account_id = google_service_account.frontend.name
  role               = "roles/iam.workloadIdentityUser"
  members            = ["principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github.name}/attribute.repository/${var.github_repo}"]
}

resource "google_service_account_iam_binding" "github_dispatcher" {
  service_account_id = google_service_account.dispatcher.name
  role               = "roles/iam.workloadIdentityUser"
  members            = ["principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github.name}/attribute.repository/${var.github_repo}"]
}

resource "google_service_account_iam_binding" "github_builder" {
  service_account_id = google_service_account.builder.name
  role               = "roles/iam.workloadIdentityUser"
  members            = ["principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github.name}/attribute.repository/${var.github_repo}"]
}

resource "google_service_account_iam_binding" "github_migration" {
  service_account_id = google_service_account.migration.name
  role               = "roles/iam.workloadIdentityUser"
  members            = ["principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github.name}/attribute.repository/${var.github_repo}"]
}