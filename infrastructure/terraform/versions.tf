terraform {
  required_version = "1.17.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "8.8.0"
    }
    google-beta = {
      source  = "hashicorp/google-beta"
      version = "8.8.0"
    }
    time = {
      source  = "hashicorp/time"
      version = "0.14.2"
    }
  }
}