# Production Environment Configuration

terraform {
  required_version = "1.17.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "8.9.0"
    }
    google-beta = {
      source  = "hashicorp/google-beta"
      version = "8.9.0"
    }
    time = {
      source  = "hashicorp/time"
      version = "0.14.2"
    }
  }

  backend "gcs" {
    bucket = "eeg-lab-terraform-state-${var.project_id}"
    prefix = "environments/prod"
  }
}

provider "google" {
  project = var.project_id
  region  = var.region
}

provider "google-beta" {
  project = var.project_id
  region  = var.region
}

provider "time" {}

# ============================================================
# Modules (Production sizing)
# ============================================================

module "network" {
  source = "../../modules/network"

  environment             = var.environment
  region                  = var.region
  private_subnet_cidr     = "10.0.0.0/20"
  cloudrun_subnet_cidr    = "10.0.16.0/20"
  vpc_connector_cidr      = "10.0.32.0/28"
  vpc_connector_min_instances = 3
  vpc_connector_max_instances = 10
  vpc_connector_machine_type  = "e2-standard-4"
}

module "service_accounts" {
  source = "../../modules/service-accounts"

  environment  = var.environment
  project_id   = var.project_id
  github_org   = var.github_org
  github_repo  = var.github_repo
}

module "cloudsql" {
  source = "../../modules/cloudsql"

  environment                   = var.environment
  region                        = var.region
  instance_tier                 = "db-custom-4-16384"
  availability_type             = "REGIONAL"
  storage_size_gb               = 100
  enable_pitr                   = true
  backup_retention_days         = 30
  max_connections               = 200
  shared_buffers                = "4GB"
  vpc_id                        = module.network.vpc_id
  vpc_name                      = module.network.vpc_name
  api_service_accounts          = [module.service_accounts.api_email]
  worker_service_accounts       = [module.service_accounts.worker_email]
  migration_service_accounts    = [module.service_accounts.migration_email]
}

module "memorystore" {
  source = "../../modules/memorystore"

  environment                  = var.environment
  region                       = var.region
  tier                         = "STANDARD_HA"
  memory_size_gb               = 5
  vpc_name                     = module.network.vpc_name
  api_service_accounts         = [module.service_accounts.api_email]
  worker_service_accounts      = [module.service_accounts.worker_email]
  dispatcher_service_accounts  = [module.service_accounts.dispatcher_email]
}

module "storage" {
  source = "../../modules/storage"

  environment                  = var.environment
  project_id                   = var.project_id
  region                       = var.region
  create_state_bucket          = false
  api_service_accounts         = [module.service_accounts.api_email]
  worker_service_accounts      = [module.service_accounts.worker_email]
  builder_service_accounts     = [module.service_accounts.builder_email]
}

module "secrets" {
  source = "../../modules/secrets"

  environment                   = var.environment
  auth_token_secret_value       = var.auth_token_secret
  db_password_value             = var.db_password
  redis_auth_value              = var.redis_auth
  oidc_client_secret_value      = var.oidc_client_secret
  api_service_accounts          = [module.service_accounts.api_email]
  worker_service_accounts       = [module.service_accounts.worker_email]
  dispatcher_service_accounts   = [module.service_accounts.dispatcher_email]
}

module "cloudrun" {
  source = "../../modules/cloudrun"

  environment                      = var.environment
  project_id                       = var.project_id
  region                           = var.region

  api_service_account_email        = module.service_accounts.api_email
  worker_service_account_email     = module.service_accounts.worker_email
  frontend_service_account_email   = module.service_accounts.frontend_email
  dispatcher_service_account_email = module.service_accounts.dispatcher_email

  vpc_network                      = module.network.vpc_name
  cloudrun_subnet                  = module.network.cloudrun_subnet_name
  vpc_connector                    = module.network.vpc_connector_name

  cloudsql_connection_name         = module.cloudsql.instance_connection_name

  redis_host                       = module.memorystore.host
  redis_port                       = module.memorystore.port

  recordings_bucket                = module.storage.recordings_bucket_name

  auth_mode                        = var.auth_mode

  api_image                        = var.api_image
  api_cpu                          = "2"
  api_memory                       = "4Gi"
  api_min_instances                = 3
  api_max_instances                = 20

  worker_image                     = var.worker_image
  worker_cpu                       = "4"
  worker_memory                    = "8Gi"
  worker_concurrency               = 8
  worker_min_instances             = 2
  worker_max_instances             = 10

  frontend_image                   = var.frontend_image
  frontend_cpu                     = "2"
  frontend_memory                  = "2Gi"
  frontend_min_instances           = 2
  frontend_max_instances           = 10

  dispatcher_image                 = var.dispatcher_image
  dispatcher_cpu                   = "1"
  dispatcher_memory                = "1Gi"
}

module "scheduler" {
  source = "../../modules/scheduler"

  environment                       = var.environment
  project_id                        = var.project_id
  project_number                    = var.project_number
  region                            = var.region
  enable_scheduler                  = true
  schedule_frequency_minutes        = 2
  dispatcher_service_account_email  = module.service_accounts.dispatcher_email
}

module "monitoring" {
  source = "../../modules/monitoring"

  environment          = var.environment
  project_id           = var.project_id
  project_number       = var.project_number
  notification_channels = var.notification_channels
  api_service_url      = module.cloudrun.api_url
}
