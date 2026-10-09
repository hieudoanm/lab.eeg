variable "project_id" {
  description = "GCP project ID"
  type        = string
}

variable "project_number" {
  description = "GCP project number"
  type        = string
}

variable "region" {
  description = "GCP region"
  type        = string
  default     = "us-central1"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "dev"
}

variable "github_org" {
  description = "GitHub organization"
  type        = string
  default     = "hieudoanm"
}

variable "github_repo" {
  description = "GitHub repository (owner/repo)"
  type        = string
  default     = "hieudoanm/eeg-lab"
}

variable "auth_mode" {
  description = "Auth mode"
  type        = string
  default     = "development"
}

variable "auth_token_secret" {
  description = "JWT signing secret (min 32 chars)"
  type        = string
  sensitive   = true
}

variable "db_password" {
  description = "Database password"
  type        = string
  sensitive   = true
  default     = ""
}

variable "redis_auth" {
  description = "Redis auth string"
  type        = string
  sensitive   = true
  default     = ""
}

variable "api_image" {
  description = "API Docker image"
  type        = string
  default     = "gcr.io/${var.project_id}/eeg-lab-api:latest"
}

variable "worker_image" {
  description = "Worker Docker image"
  type        = string
  default     = "gcr.io/${var.project_id}/eeg-lab-worker:latest"
}

variable "frontend_image" {
  description = "Frontend Docker image"
  type        = string
  default     = "gcr.io/${var.project_id}/eeg-lab-frontend:latest"
}

variable "dispatcher_image" {
  description = "Dispatcher Docker image"
  type        = string
  default     = "gcr.io/${var.project_id}/eeg-lab-dispatcher:latest"
}

variable "notification_channels" {
  description = "Notification channel IDs for alerts"
  type        = list(string)
  default     = []
}
