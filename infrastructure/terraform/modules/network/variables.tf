variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  validation {
    condition     = contains(["dev", "staging", "prod"], var.environment)
    error_message = "Environment must be dev, staging, or prod."
  }
}

variable "region" {
  description = "GCP region for resources"
  type        = string
  default     = "us-central1"
}

variable "private_subnet_cidr" {
  description = "CIDR range for private subnet (Cloud SQL, Redis)"
  type        = string
  default     = "10.0.0.0/20"
  validation {
    condition     = can(regex("^10\\.\\d+\\.\\d+\\.0/20$", var.private_subnet_cidr))
    error_message = "Private subnet must be a /20 in 10.x.x.0/20 range."
  }
}

variable "cloudrun_subnet_cidr" {
  description = "CIDR range for Cloud Run subnet"
  type        = string
  default     = "10.0.16.0/20"
  validation {
    condition     = can(regex("^10\\.\\d+\\.\\d+\\.0/20$", var.cloudrun_subnet_cidr))
    error_message = "Cloud Run subnet must be a /20 in 10.x.x.0/20 range."
  }
}

variable "vpc_connector_cidr" {
  description = "CIDR range for Serverless VPC Access connector (/28)"
  type        = string
  default     = "10.0.32.0/28"
  validation {
    condition     = can(regex("^10\\.\\d+\\.\\d+\\.0/28$", var.vpc_connector_cidr))
    error_message = "VPC connector CIDR must be a /28 in 10.x.x.0/28 range."
  }
}

variable "vpc_connector_min_instances" {
  description = "Minimum instances for VPC connector"
  type        = number
  default     = 2
  validation {
    condition     = var.vpc_connector_min_instances >= 2 && var.vpc_connector_min_instances <= 10
    error_message = "VPC connector min instances must be between 2 and 10."
  }
}

variable "vpc_connector_max_instances" {
  description = "Maximum instances for VPC connector"
  type        = number
  default     = 10
  validation {
    condition     = var.vpc_connector_max_instances >= var.vpc_connector_min_instances && var.vpc_connector_max_instances <= 100
    error_message = "VPC connector max instances must be >= min and <= 100."
  }
}

variable "vpc_connector_machine_type" {
  description = "Machine type for VPC connector"
  type        = string
  default     = "e2-micro"
  validation {
    condition     = contains(["f1-micro", "e2-micro", "e2-standard-4"], var.vpc_connector_machine_type)
    error_message = "VPC connector machine type must be f1-micro, e2-micro, or e2-standard-4."
  }
}