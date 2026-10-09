variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
}

variable "region" {
  description = "GCP region"
  type        = string
  default     = "us-central1"
}

variable "instance_tier" {
  description = "Cloud SQL machine tier"
  type        = string
  default     = "db-f1-micro"
  validation {
    condition     = contains(["db-f1-micro", "db-g1-small", "db-custom-2-8192", "db-custom-4-16384", "db-custom-8-32768"], var.instance_tier)
    error_message = "Invalid instance tier."
  }
}

variable "availability_type" {
  description = "Availability type (ZONAL or REGIONAL for HA)"
  type        = string
  default     = "ZONAL"
  validation {
    condition     = contains(["ZONAL", "REGIONAL"], var.availability_type)
    error_message = "Availability type must be ZONAL or REGIONAL."
  }
}

variable "storage_size_gb" {
  description = "Initial storage size in GB"
  type        = number
  default     = 10
  validation {
    condition     = var.storage_size_gb >= 10 && var.storage_size_gb <= 65536
    error_message = "Storage size must be between 10 and 65536 GB."
  }
}

variable "enable_pitr" {
  description = "Enable point-in-time recovery"
  type        = bool
  default     = true
}

variable "backup_retention_days" {
  description = "Number of days to retain backups"
  type        = number
  default     = 7
  validation {
    condition     = var.backup_retention_days >= 1 && var.backup_retention_days <= 365
    error_message = "Backup retention must be between 1 and 365 days."
  }
}

variable "max_connections" {
  description = "Max PostgreSQL connections"
  type        = number
  default     = 100
}

variable "shared_buffers" {
  description = "PostgreSQL shared_buffers (e.g., '128MB')"
  type        = string
  default     = "128MB"
}

variable "enable_private_service_connect" {
  description = "Enable Private Service Connect for direct private IP"
  type        = bool
  default     = false
}

variable "vpc_id" {
  description = "VPC network ID"
  type        = string
}

variable "vpc_name" {
  description = "VPC network name"
  type        = string
}

variable "api_service_accounts" {
  description = "Service accounts that need Cloud SQL client access (API)"
  type        = list(string)
  default     = []
}

variable "worker_service_accounts" {
  description = "Service accounts that need Cloud SQL client access (Worker)"
  type        = list(string)
  default     = []
}

variable "migration_service_accounts" {
  description = "Service accounts that need Cloud SQL client access (Migrations)"
  type        = list(string)
  default     = []
}