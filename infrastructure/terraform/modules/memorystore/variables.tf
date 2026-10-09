variable "environment" {
  description = "Environment name"
  type        = string
}

variable "region" {
  description = "GCP region"
  type        = string
  default     = "us-central1"
}

variable "tier" {
  description = "Redis tier (BASIC, STANDARD_HA)"
  type        = string
  default     = "BASIC"
  validation {
    condition     = contains(["BASIC", "STANDARD_HA"], var.tier)
    error_message = "Tier must be BASIC or STANDARD_HA."
  }
}

variable "memory_size_gb" {
  description = "Redis memory size in GB"
  type        = number
  default     = 1
  validation {
    condition     = var.memory_size_gb >= 1 && var.memory_size_gb <= 300
    error_message = "Memory size must be between 1 and 300 GB."
  }
}

variable "vpc_name" {
  description = "VPC network name"
  type        = string
}

variable "api_service_accounts" {
  description = "Service accounts with viewer access (API)"
  type        = list(string)
  default     = []
}

variable "worker_service_accounts" {
  description = "Service accounts with editor access (Worker)"
  type        = list(string)
  default     = []
}

variable "dispatcher_service_accounts" {
  description = "Service accounts with editor access (Dispatcher)"
  type        = list(string)
  default     = []
}