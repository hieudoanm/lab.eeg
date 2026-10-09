output "api_url" {
  description = "API service URL"
  value       = google_cloud_run_v2_service.api.uri
}

output "api_name" {
  description = "API service name"
  value       = google_cloud_run_v2_service.api.name
}

output "worker_name" {
  description = "Worker service name"
  value       = google_cloud_run_v2_service.worker.name
}

output "frontend_url" {
  description = "Frontend service URL"
  value       = google_cloud_run_v2_service.frontend.uri
}

output "frontend_name" {
  description = "Frontend service name"
  value       = google_cloud_run_v2_service.frontend.name
}

output "dispatcher_job_name" {
  description = "Dispatcher job name"
  value       = google_cloud_run_v2_job.dispatcher.name
}