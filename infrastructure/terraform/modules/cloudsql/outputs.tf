output "instance_name" {
  description = "Cloud SQL instance name"
  value       = google_sql_database_instance.primary.name
}

output "instance_connection_name" {
  description = "Cloud SQL connection name (project:region:instance)"
  value       = google_sql_database_instance.primary.connection_name
}

output "instance_private_ip" {
  description = "Private IP address of the instance"
  value       = google_sql_database_instance.primary.private_ip_address
}

output "instance_public_ip" {
  description = "Public IP address (empty if not enabled)"
  value       = google_sql_database_instance.primary.public_ip_address
}

output "database_name" {
  description = "Database name"
  value       = google_sql_database.neurolab.name
}

output "database_user" {
  description = "Database IAM user"
  value       = google_sql_user.neurolab.name
}

output "private_service_connect_ip" {
  description = "Private Service Connect IP (if enabled)"
  value       = var.enable_private_service_connect ? google_compute_address.private_ip[0].address : null
}