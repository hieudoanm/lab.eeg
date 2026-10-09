output "vpc_id" {
  description = "VPC network ID"
  value       = google_compute_network.vpc.id
}

output "vpc_name" {
  description = "VPC network name"
  value       = google_compute_network.vpc.name
}

output "private_subnet_id" {
  description = "Private subnet ID (for Cloud SQL, Redis)"
  value       = google_compute_subnetwork.private.id
}

output "private_subnet_name" {
  description = "Private subnet name"
  value       = google_compute_subnetwork.private.name
}

output "cloudrun_subnet_id" {
  description = "Cloud Run subnet ID"
  value       = google_compute_subnetwork.cloudrun.id
}

output "cloudrun_subnet_name" {
  description = "Cloud Run subnet name"
  value       = google_compute_subnetwork.cloudrun.name
}

output "vpc_connector_id" {
  description = "Serverless VPC Access connector ID"
  value       = google_vpc_access_connector.connector.id
}

output "vpc_connector_name" {
  description = "Serverless VPC Access connector name"
  value       = google_vpc_access_connector.connector.name
}

output "private_dns_zone_name" {
  description = "Private DNS zone name"
  value       = google_dns_managed_zone.private.name
}