# Cloud Scheduler for Outbox Dispatcher

resource "google_cloud_scheduler_job" "dispatcher" {
  count = var.enable_scheduler ? 1 : 0

  name        = "neurolab-${var.environment}-dispatcher"
  description = "Runs outbox dispatcher every ${var.schedule_frequency_minutes} minutes"
  schedule    = "*/${var.schedule_frequency_minutes} * * * *"
  time_zone   = "UTC"
  region      = var.region

  http_target {
    http_method = "POST"
    uri         = "https://${var.region}-run.googleapis.com/apis/run.googleapis.com/v1/namespaces/${var.project_id}/jobs/neurolab-${var.environment}-dispatcher:run"
    oidc_token {
      service_account_email = var.dispatcher_service_account_email
    }
  }

  retry_config {
    retry_count = 3
    max_backoff_duration = "60s"
    min_backoff_duration = "10s"
    max_doublings = 2
  }

  attempt_deadline = "300s"
}

# Allow Cloud Scheduler to invoke the dispatcher job
resource "google_cloud_run_v2_job_iam_binding" "scheduler_invoker" {
  count = var.enable_scheduler ? 1 : 0
  job    = "neurolab-${var.environment}-dispatcher"
  location = var.region
  role   = "roles/run.invoker"
  members = ["serviceAccount:service-${var.project_number}@gcp-sa-cloudscheduler.iam.gserviceaccount.com"]
}