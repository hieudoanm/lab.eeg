output "recordings_bucket_name" {
  description = "Recordings bucket name"
  value       = google_storage_bucket.recordings.name
}

output "recordings_bucket_url" {
  description = "Recordings bucket gs:// URL"
  value       = "gs://${google_storage_bucket.recordings.name}"
}

output "artifacts_bucket_name" {
  description = "Artifacts bucket name"
  value       = google_storage_bucket.artifacts.name
}

output "artifacts_bucket_url" {
  description = "Artifacts bucket gs:// URL"
  value       = "gs://${google_storage_bucket.artifacts.name}"
}

output "terraform_state_bucket_name" {
  description = "Terraform state bucket name (if created)"
  value       = var.create_state_bucket ? google_storage_bucket.terraform_state[0].name : null
}