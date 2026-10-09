output "instance_id" {
  description = "Redis instance ID"
  value       = google_redis_instance.primary.id
}

output "instance_name" {
  description = "Redis instance name"
  value       = google_redis_instance.primary.name
}

output "host" {
  description = "Redis host (internal IP)"
  value       = google_redis_instance.primary.host
}

output "port" {
  description = "Redis port"
  value       = google_redis_instance.primary.port
}

output "auth_string" {
  description = "Redis AUTH string (sensitive)"
  value       = google_redis_instance.primary.auth_string
  sensitive   = true
}

output "connection_string" {
  description = "Redis connection string (rediss://)"
  value       = "rediss://:${google_redis_instance.primary.auth_string}@${google_redis_instance.primary.host}:${google_redis_instance.primary.port}"
  sensitive   = true
}