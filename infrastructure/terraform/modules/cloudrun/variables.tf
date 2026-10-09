variable "environment" {
  description = "Environment name"
  type        = string
}

variable "project_id" {
  description = "GCP project ID"
  type        = string
}

variable "region" {
  description = "GCP region"
  type        = string
  default     = "us-central1"
}

# Service account emails
variable "api_service_account_email" {
  description = "API service account email"
  type        = string
}
variable "worker_service_account_email" {
  description = "Worker service account email"
  type        = string
}
variable "frontend_service_account_email" {
  description = "Frontend service account email"
  type        = string
}
variable "dispatcher_service_account_email" {
  description = "Dispatcher service account email"
  type        = string
}

# VPC
variable "vpc_network" {
  description = "VPC network name"
  type        = string
}
variable "cloudrun_subnet" {
  description = "Cloud Run subnet name"
  type        = string
}
variable "vpc_connector" {
  description = "Serverless VPC Access connector name"
  type        = string
}

# Cloud SQL
variable "cloudsql_connection_name" {
  description = "Cloud SQL connection name (project:region:instance)"
  type        = string
}

# Redis
variable "redis_host" {
  description = "Redis host"
  type        = string
}
variable "redis_port" {
  description = "Redis port"
  type        = number
  default     = 6379
}

# Storage
variable "recordings_bucket" {
  description = "Recordings bucket name"
  type        = string
}

# Auth
variable "auth_mode" {
  description = "Auth mode (development, oidc)"
  type        = string
  default     = "development"
}

# API
variable "api_image" {
  description = "API Docker image"
  type        = string
}
variable "api_cpu" {
  description = "API CPU limit"
  type        = string
  default     = "2"
}
variable "api_memory" {
  description = "API memory limit"
  type        = string
  default     = "2Gi"
}
variable "api_min_instances" {
  description = "API minimum instances"
  type        = number
  default     = 1
}
variable "api_max_instances" {
  description = "API maximum instances"
  type        = number
  default     = 10
}

# Worker
variable "worker_image" {
  description = "Worker Docker image"
  type        = string
}
variable "worker_cpu" {
  description = "Worker CPU limit"
  type        = string
  default     = "2"
}
variable "worker_memory" {
  description = "Worker memory limit"
  type        = string
  default     = "4Gi"
}
variable "worker_concurrency" {
  description = "Celery worker concurrency"
  type        = number
  default     = 4
}
variable "worker_min_instances" {
  description = "Worker minimum instances"
  type        = number
  default     = 0
}
variable "worker_max_instances" {
  description = "Worker maximum instances"
  type        = number
  default     = 5
}

# Frontend
variable "frontend_image" {
  description = "Frontend Docker image"
  type        = string
}
variable "frontend_cpu" {
  description = "Frontend CPU limit"
  type        = string
  default     = "1"
}
variable "frontend_memory" {
  description = "Frontend memory limit"
  type        = string
  default     = "1Gi"
}
variable "frontend_min_instances" {
  description = "Frontend minimum instances"
  type        = number
  default     = 1
}
variable "frontend_max_instances" {
  description = "Frontend maximum instances"
  type        = number
  default     = 5
}

# Dispatcher
variable "dispatcher_image" {
  description = "Dispatcher Docker image"
  type        = string
}
variable "dispatcher_cpu" {
  description = "Dispatcher CPU limit"
  type        = string
  default     = "1"
}
variable "dispatcher_memory" {
  description = "Dispatcher memory limit"
  type        = string
  default     = "1Gi"
}