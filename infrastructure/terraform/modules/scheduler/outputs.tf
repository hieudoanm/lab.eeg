output "scheduler_job_name" {
  description = "Cloud Scheduler job name"
  value       = var.enable_scheduler ? google_cloud_scheduler_job.dispatcher[0].name : null
}