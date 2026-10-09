# Memorystore Redis Instance

resource "google_redis_instance" "primary" {
  name           = "eeg-lab-${var.environment}-redis"
  region         = var.region
  redis_version  = "REDIS_7_2"
  tier           = var.tier
  memory_size_gb = var.memory_size_gb

  authorized_network = var.vpc_name

  redis_configs = {
    maxmemory-policy          = "allkeys-lru"
    notify-keyspace-events    = "Ex"
    slowlog-log-slower-than   = "10000"
    slowlog-max-len           = "128"
    timeout                   = "300"
  }

  maintenance_policy {
    weekly_maintenance_window {
      day = "SUNDAY"
      duration = "3600s"
      start_time = "04:00"
    }
  }

  replica_count = var.tier == "STANDARD_HA" ? 1 : 0

  transit_encryption_mode = "SERVER_AUTHENTICATION"
  auth_enabled            = true

  deletion_protection = var.environment == "prod"
}

# IAM bindings for service accounts
resource "google_redis_instance_iam_binding" "api_client" {
  instance = google_redis_instance.primary.id
  role     = "roles/redis.viewer"
  members  = var.api_service_accounts
}

resource "google_redis_instance_iam_binding" "worker_client" {
  instance = google_redis_instance.primary.id
  role     = "roles/redis.editor" # Worker needs to publish/consume
  members  = var.worker_service_accounts
}

resource "google_redis_instance_iam_binding" "dispatcher_client" {
  instance = google_redis_instance.primary.id
  role     = "roles/redis.editor"
  members  = var.dispatcher_service_accounts
}
