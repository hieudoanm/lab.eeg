# Cloud SQL PostgreSQL Instance

resource "google_sql_database_instance" "primary" {
  name             = "eeg-lab-${var.environment}-db"
  region           = var.region
  database_version = "POSTGRES_16"

  settings {
    tier              = var.instance_tier
    availability_type = var.availability_type
    storage_auto_resize = true
    storage_size      = var.storage_size_gb
    storage_type      = "SSD"

    backup_configuration {
      enabled            = true
      start_time         = "03:00"
      location           = var.region
      point_in_time_recovery = var.enable_pitr
      backup_retention_settings {
        retained_backups = var.backup_retention_days
      }
    }

    ip_configuration {
      private_network = var.vpc_name
      require_ssl     = true
      authorized_networks = [] # No public IP
    }

    database_flags {
      name  = "cloudsql.iam_authentication"
      value = "on"
    }

    database_flags {
      name  = "max_connections"
      value = var.max_connections
    }

    database_flags {
      name  = "shared_buffers"
      value = var.shared_buffers
    }

    insights_config {
      query_insights_enabled = true
      query_string_length    = 1024
      query_plans_per_minute = 10
    }

    maintenance_window {
      day  = 7 # Sunday
      hour = 4
    }
  }

  deletion_protection = var.environment == "prod"
}

# Database and user
resource "google_sql_database" "eeg-lab" {
  name     = "eeg-lab"
  instance = google_sql_database_instance.primary.name
  charset  = "UTF8"
  collation = "en_US.UTF8"
}

resource "google_sql_user" "eeg-lab" {
  name     = "eeg-lab"
  instance = google_sql_database_instance.primary.name
  type     = "CLOUD_IAM_USER" # Use IAM authentication
}

# IAM binding for service accounts to use Cloud SQL
resource "google_sql_database_instance_iam_binding" "api_client" {
  instance = google_sql_database_instance.primary.name
  role     = "roles/cloudsql.client"
  members  = var.api_service_accounts
}

resource "google_sql_database_instance_iam_binding" "worker_client" {
  instance = google_sql_database_instance.primary.name
  role     = "roles/cloudsql.client"
  members  = var.worker_service_accounts
}

resource "google_sql_database_instance_iam_binding" "migration_client" {
  instance = google_sql_database_instance.primary.name
  role     = "roles/cloudsql.client"
  members  = var.migration_service_accounts
}

# Private Service Connect (optional, for direct private IP access)
resource "google_compute_address" "private_ip" {
  count = var.enable_private_service_connect ? 1 : 0
  name  = "eeg-lab-${var.environment}-cloudsql-ip"
  purpose = "VPC_PEERING"
  address_type = "INTERNAL"
  prefix_length = 16
  network = var.vpc_id
}
