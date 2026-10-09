# VPC Network with Private Service Connect for Cloud SQL and Memorystore

resource "google_compute_network" "vpc" {
  name                    = "eeg-lab-${var.environment}-vpc"
  auto_create_subnetworks = false
  description             = "EEG Lab ${var.environment} VPC"
  routing_mode            = "GLOBAL"
  deletion_policy         = "DELETE"
}

resource "google_compute_subnetwork" "private" {
  name          = "eeg-lab-${var.environment}-private"
  network       = google_compute_network.vpc.id
  ip_cidr_range = var.private_subnet_cidr
  region        = var.region
  purpose       = "PRIVATE"
  private_ip_google_access = true

  log_config {
    aggregation_interval = "INTERVAL_10_MIN"
    flow_sampling        = 0.5
    metadata             = "INCLUDE_ALL_METADATA"
  }
}

resource "google_compute_subnetwork" "cloudrun" {
  name          = "eeg-lab-${var.environment}-cloudrun"
  network       = google_compute_network.vpc.id
  ip_cidr_range = var.cloudrun_subnet_cidr
  region        = var.region
  purpose       = "PRIVATE"
  role          = "ACTIVE"
  private_ip_google_access = true
}

# Cloud NAT for egress from private subnets (Cloud Run, Cloud Build, etc.)
resource "google_compute_router" "nat" {
  name    = "eeg-lab-${var.environment}-router"
  network = google_compute_network.vpc.id
  region  = var.region
}

resource "google_compute_router_nat" "nat" {
  name                               = "eeg-lab-${var.environment}-nat"
  router                             = google_compute_router.nat.name
  region                             = var.region
  nat_ip_allocate_option             = "AUTO_ONLY"
  source_subnetwork_ip_ranges_to_nat = "ALL_SUBNETWORKS_ALL_IP_RANGES"
  log_config {
    enable = true
    filter = "ERRORS_ONLY"
  }
}

# Private Service Connect for Cloud SQL and Memorystore
resource "google_compute_global_address" "private_ip" {
  name          = "eeg-lab-${var.environment}-private-ip"
  purpose       = "VPC_PEERING"
  address_type  = "INTERNAL"
  prefix_length = 16
  network       = google_compute_network.vpc.id
}

resource "google_service_networking_connection" "private_psc" {
  network                 = google_compute_network.vpc.id
  service                 = "servicenetworking.googleapis.com"
  reserved_peering_ranges = [google_compute_global_address.private_ip.name]
}

# Firewall rules
resource "google_compute_firewall" "allow_internal" {
  name    = "eeg-lab-${var.environment}-allow-internal"
  network = google_compute_network.vpc.name
  direction = "INGRESS"
  priority  = 1000

  allow {
    protocol = "tcp"
    ports    = ["0-65535"]
  }
  allow {
    protocol = "udp"
    ports    = ["0-65535"]
  }
  allow {
    protocol = "icmp"
  }

  source_ranges = [var.private_subnet_cidr, var.cloudrun_subnet_cidr]
  description   = "Allow all traffic within private subnets"
}

resource "google_compute_firewall" "allow_health_checks" {
  name    = "eeg-lab-${var.environment}-allow-health-checks"
  network = google_compute_network.vpc.name
  direction = "INGRESS"
  priority  = 1000

  allow {
    protocol = "tcp"
    ports    = ["8080", "8000", "3000"]
  }

  source_ranges = [
    "35.191.0.0/16",   # Google health checks
    "130.211.0.0/22",  # Google health checks
  ]
  description = "Allow GCP health checks to Cloud Run"
}

resource "google_compute_firewall" "allow_vpc_connector" {
  name    = "eeg-lab-${var.environment}-allow-vpc-connector"
  network = google_compute_network.vpc.name
  direction = "INGRESS"
  priority  = 1000

  allow {
    protocol = "tcp"
    ports    = ["0-65535"]
  }
  allow {
    protocol = "udp"
    ports    = ["0-65535"]
  }

  source_ranges = [var.vpc_connector_cidr]
  description   = "Allow traffic from Serverless VPC Access connector"
}

# VPC Access Connector for Cloud Run to reach private services
resource "google_vpc_access_connector" "connector" {
  name           = "eeg-lab-${var.environment}-connector"
  region         = var.region
  network        = google_compute_network.vpc.id
  subnet         = google_compute_subnetwork.cloudrun.id
  min_instances  = var.vpc_connector_min_instances
  max_instances  = var.vpc_connector_max_instances
  machine_type   = var.vpc_connector_machine_type
}

# DNS zone for internal service discovery (optional)
resource "google_dns_managed_zone" "private" {
  name        = "eeg-lab-${var.environment}-private-zone"
  dns_name    = "eeg-lab.internal."
  description = "Private DNS zone for EEG Lab ${var.environment}"
  visibility  = "private"
  private_visibility_config {
    networks {
      network_url = google_compute_network.vpc.id
    }
  }
}
