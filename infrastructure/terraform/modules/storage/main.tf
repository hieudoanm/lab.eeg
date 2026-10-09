# Cloud Storage Buckets

# Recordings bucket (private, versioned)
resource "google_storage_bucket" "recordings" {
  name          = "neurolab-${var.environment}-recordings-${var.project_id}"
  location      = var.region
  storage_class = "STANDARD"
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"

  versioning {
    enabled = true
  }

  lifecycle_rule {
    action {
      type = "Delete"
    }
    condition {
      age                    = 90
      is_live                = false
      num_newer_versions     = 3
    }
  }

  lifecycle_rule {
    action {
      type = "AbortIncompleteMultipartUpload"
    }
    condition {
      age = 7
    }
  }

  labels = {
    environment = var.environment
    purpose     = "recordings"
  }
}

# Artifacts bucket (Docker images, build artifacts)
resource "google_storage_bucket" "artifacts" {
  name          = "neurolab-${var.environment}-artifacts-${var.project_id}"
  location      = var.region
  storage_class = "STANDARD"
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"

  lifecycle_rule {
    action {
      type = "Delete"
    }
    condition {
      age = 30
    }
  }

  labels = {
    environment = var.environment
    purpose     = "artifacts"
  }
}

# Terraform state bucket (created separately, but can be managed here)
resource "google_storage_bucket" "terraform_state" {
  count = var.create_state_bucket ? 1 : 0
  name  = "neurolab-terraform-state-${var.project_id}"
  location = var.region
  storage_class = "STANDARD"
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"

  versioning {
    enabled = true
  }

  lifecycle_rule {
    action {
      type = "Delete"
    }
    condition {
      is_live = false
      num_newer_versions = 10
    }
  }

  labels = {
    environment = var.environment
    purpose     = "terraform-state"
  }
}

# IAM bindings for service accounts
resource "google_storage_bucket_iam_binding" "recordings_api" {
  bucket = google_storage_bucket.recordings.name
  role   = "roles/storage.objectUser"
  members = var.api_service_accounts
}

resource "google_storage_bucket_iam_binding" "recordings_worker" {
  bucket = google_storage_bucket.recordings.name
  role   = "roles/storage.objectUser"
  members = var.worker_service_accounts
}

resource "google_storage_bucket_iam_binding" "artifacts_builder" {
  bucket = google_storage_bucket.artifacts.name
  role   = "roles/storage.objectCreator"
  members = var.builder_service_accounts
}

resource "google_storage_bucket_iam_binding" "artifacts_api" {
  bucket = google_storage_bucket.artifacts.name
  role   = "roles/storage.objectViewer"
  members = var.api_service_accounts
}