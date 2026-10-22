terraform {
  required_version = "1.16.6"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "8.7.0"
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