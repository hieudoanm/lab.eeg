output "api_email" {
  description = "API service account email"
  value       = google_service_account.api.email
}

output "worker_email" {
  description = "Worker service account email"
  value       = google_service_account.worker.email
}

output "frontend_email" {
  description = "Frontend service account email"
  value       = google_service_account.frontend.email
}

output "dispatcher_email" {
  description = "Dispatcher service account email"
  value       = google_service_account.dispatcher.email
}

output "builder_email" {
  description = "Builder service account email"
  value       = google_service_account.builder.email
}

output "migration_email" {
  description = "Migration service account email"
  value       = google_service_account.migration.email
}

output "workload_identity_pool_name" {
  description = "Workload Identity Pool name"
  value       = google_iam_workload_identity_pool.github.name
}

output "workload_identity_pool_provider_name" {
  description = "Workload Identity Pool Provider name"
  value       = google_iam_workload_identity_pool_provider.github.name
}