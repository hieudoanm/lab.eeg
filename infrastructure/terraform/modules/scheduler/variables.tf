variable "environment" {
  description = "Environment name"
  type        = string
}

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

variable "enable_scheduler" {
  description = "Enable Cloud Scheduler job for dispatcher"
  type        = bool
  default     = true
}

variable "schedule_frequency_minutes" {
  description = "Scheduler frequency in minutes"
  type        = number
  default     = 30
  validation {
    condition     = var.schedule_frequency_minutes >= 1 && var.schedule_frequency_minutes <= 60
    error_message = "Schedule frequency must be between 1 and 60 minutes."
  }
}

variable "dispatcher_service_account_email" {
  description = "Dispatcher service account email"
  type        = string
}