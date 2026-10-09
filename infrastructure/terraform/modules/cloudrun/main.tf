# Cloud Run Services

# Shared Cloud Run service configuration
locals {
  common_labels = {
    environment = var.environment
    project     = var.project_id
    managed-by  = "terraform"
  }

  common_annotations = {
    "run.googleapis.com/client-name" = "terraform"
  }
}

# ============================================================
# API Service
# ============================================================
resource "google_cloud_run_v2_service" "api" {
  name     = "eeg-lab-${var.environment}-api"
  location = var.region

  template {
    service_account = var.api_service_account_email
    timeout         = "300s"
    min_instance_count = var.api_min_instances
    max_instance_count = var.api_max_instances

    containers {
      image = var.api_image
      ports {
        name          = "http1"
        container_port = 8000
      }
      env {
        name  = "APP_ENV"
        value = var.environment
      }
      env {
        name  = "DATABASE_URL"
        value = "postgresql://eeg-lab@${var.cloudsql_connection_name}/eeg-lab?host=/cloudsql/${var.cloudsql_connection_name}"
      }
      env {
        name  = "REDIS_URL"
        value = "rediss://${var.redis_host}:${var.redis_port}"
      }
      env {
        name  = "AUTH_MODE"
        value = var.auth_mode
      }
      env {
        name  = "LOG_FORMAT"
        value = "json"
      }
      env {
        name  = "STORAGE_BACKEND"
        value = "gcs"
      }
      env {
        name  = "GCS_BUCKET"
        value = var.recordings_bucket
      }
      resources {
        limits = {
          cpu    = var.api_cpu
          memory = var.api_memory
        }
      }
      startup_probe {
        initial_delay_seconds = 10
        period_seconds        = 10
        timeout_seconds       = 5
        failure_threshold     = 30
        http_get {
          path = "/health/live"
          port = 8000
        }
      }
      liveness_probe {
        initial_delay_seconds = 30
        period_seconds        = 30
        timeout_seconds       = 10
        failure_threshold     = 3
        http_get {
          path = "/health/live"
          port = 8000
        }
      }
    }

    vpc_access {
      egress           = "ALL_TRAFFIC"
      connector        = var.vpc_connector
      network_interfaces {
        network = var.vpc_network
        subnetwork = var.cloudrun_subnet
      }
    }

    scaling {
      min_instance_count = var.api_min_instances
      max_instance_count = var.api_max_instances
    }
  }

  traffics {
    percent         = 100
    latest_revision = true
  }

  labels = local.common_labels
}

# API IAM - allow unauthenticated for health checks, authenticated for API
resource "google_cloud_run_v2_service_iam_binding" "api_public_health" {
  service = google_cloud_run_v2_service.api.name
  location = var.region
  role    = "roles/run.invoker"
  members = ["allUsers"]
  condition {
    title       = "Allow unauthenticated health checks"
    expression  = 'request.path.startsWith("/health/")'
  }
}

# ============================================================
# Worker Service
# ============================================================
resource "google_cloud_run_v2_service" "worker" {
  name     = "eeg-lab-${var.environment}-worker"
  location = var.region

  template {
    service_account = var.worker_service_account_email
    timeout         = "600s"
    min_instance_count = var.worker_min_instances
    max_instance_count = var.worker_max_instances

    containers {
      image = var.worker_image
      command = ["celery", "-A", "app.workers.celery_app", "worker", "--loglevel=INFO", "--concurrency=${var.worker_concurrency}"]
      env {
        name  = "APP_ENV"
        value = var.environment
      }
      env {
        name  = "DATABASE_URL"
        value = "postgresql://eeg-lab@${var.cloudsql_connection_name}/eeg-lab?host=/cloudsql/${var.cloudsql_connection_name}"
      }
      env {
        name  = "REDIS_URL"
        value = "rediss://${var.redis_host}:${var.redis_port}"
      }
      env {
        name  = "LOG_FORMAT"
        value = "json"
      }
      env {
        name  = "STORAGE_BACKEND"
        value = "gcs"
      }
      env {
        name  = "GCS_BUCKET"
        value = var.recordings_bucket
      }
      resources {
        limits = {
          cpu    = var.worker_cpu
          memory = var.worker_memory
        }
      }
    }

    vpc_access {
      egress           = "ALL_TRAFFIC"
      connector        = var.vpc_connector
      network_interfaces {
        network = var.vpc_network
        subnetwork = var.cloudrun_subnet
      }
    }

    scaling {
      min_instance_count = var.worker_min_instances
      max_instance_count = var.worker_max_instances
    }
  }

  labels = local.common_labels
}

# ============================================================
# Frontend Service
# ============================================================
resource "google_cloud_run_v2_service" "frontend" {
  name     = "eeg-lab-${var.environment}-frontend"
  location = var.region

  template {
    service_account = var.frontend_service_account_email
    timeout         = "300s"
    min_instance_count = var.frontend_min_instances
    max_instance_count = var.frontend_max_instances

    containers {
      image = var.frontend_image
      ports {
        name          = "http1"
        container_port = 3000
      }
      env {
        name  = "NEXT_PUBLIC_API_BASE_URL"
        value = "https://${google_cloud_run_v2_service.api.uri}"
      }
      resources {
        limits = {
          cpu    = var.frontend_cpu
          memory = var.frontend_memory
        }
      }
      startup_probe {
        initial_delay_seconds = 10
        period_seconds        = 10
        timeout_seconds       = 5
        failure_threshold     = 30
        http_get {
          path = "/"
          port = 3000
        }
      }
    }

    vpc_access {
      egress = "PRIVATE_RANGES_ONLY"
    }

    scaling {
      min_instance_count = var.frontend_min_instances
      max_instance_count = var.frontend_max_instances
    }
  }

  traffics {
    percent         = 100
    latest_revision = true
  }

  labels = local.common_labels
}

resource "google_cloud_run_v2_service_iam_binding" "frontend_public" {
  service  = google_cloud_run_v2_service.frontend.name
  location = var.region
  role     = "roles/run.invoker"
  members  = ["allUsers"]
}

# ============================================================
# Dispatcher Service (runs as a job, not a service)
# ============================================================
resource "google_cloud_run_v2_job" "dispatcher" {
  name     = "eeg-lab-${var.environment}-dispatcher"
  location = var.region

  template {
    template {
      service_account = var.dispatcher_service_account_email
      timeout         = "300s"
      max_retries     = 3

      containers {
        image = var.dispatcher_image
        command = ["python", "-m", "app.workers.outbox_dispatcher", "--batch-size=100", "--max-batches=20"]
        env {
          name  = "APP_ENV"
          value = var.environment
        }
        env {
          name  = "DATABASE_URL"
          value = "postgresql://eeg-lab@${var.cloudsql_connection_name}/eeg-lab?host=/cloudsql/${var.cloudsql_connection_name}"
        }
        env {
          name  = "REDIS_URL"
          value = "rediss://${var.redis_host}:${var.redis_port}"
        }
        env {
          name  = "LOG_FORMAT"
          value = "json"
        }
        resources {
          limits = {
            cpu    = var.dispatcher_cpu
            memory = var.dispatcher_memory
          }
        }
      }

      vpc_access {
        egress           = "ALL_TRAFFIC"
        connector        = var.vpc_connector
        network_interfaces {
          network = var.vpc_network
          subnetwork = var.cloudrun_subnet
        }
      }
    }
  }

  labels = local.common_labels
}
