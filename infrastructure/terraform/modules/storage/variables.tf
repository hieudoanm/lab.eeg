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

variable "create_state_bucket" {
  description = "Create Terraform state bucket (run once per project)"
  type        = bool
  default     = false
}

variable "api_service_accounts" {
  description = "Service accounts with object access (API)"
  type        = list(string)
  default     = []
}

variable "worker_service_accounts" {
  description = "Service accounts with object access (Worker)"
  type        = list(string)
  default     = []
}

variable "builder_service_accounts" {
  description = "Service accounts with object creator access (Cloud Build)"
  type        = list(string)
  default     = []
}