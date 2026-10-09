# Monitoring: Log-based Metrics, Alert Policies, and Dashboards

# Log-based metric: HTTP request latency
resource "google_logging_metric" "http_request_duration" {
  name   = "eeg-lab-${var.environment}-http-request-duration"
  description = "HTTP request latency from API structured logs"
  filter = <<-EOT
    resource.type="cloud_run_revision"
    resource.labels.service_name="eeg-lab-${var.environment}-api"
    severity>=DEFAULT
    jsonPayload.duration_ms>0
  EOT

  metric_descriptor {
    metric_kind = "DELTA"
    value_type  = "DISTRIBUTION"
    unit        = "ms"
    labels {
      key         = "method"
      value_type  = "STRING"
      description = "HTTP method"
    }
    labels {
      key         = "path"
      value_type  = "STRING"
      description = "Request path"
    }
    labels {
      key         = "status"
      value_type  = "STRING"
      description = "HTTP status code"
    }
    bucket_options {
      exponential_buckets {
        num_finite_buckets = 20
        growth_factor      = 2.0
        scale              = 1.0
      }
    }
  }

  label_extractors = {
    "method" = 'EXTRACT(jsonPayload.method)'
    "path"   = 'EXTRACT(jsonPayload.route)'
    "status" = 'EXTRACT(jsonPayload.status_code)'
  }

  value_extractor = 'EXTRACT(jsonPayload.duration_ms)'
}

# Log-based metric: HTTP request count
resource "google_logging_metric" "http_request_count" {
  name   = "eeg-lab-${var.environment}-http-request-count"
  description = "HTTP request count by method, path, status"
  filter = <<-EOT
    resource.type="cloud_run_revision"
    resource.labels.service_name="eeg-lab-${var.environment}-api"
    jsonPayload.status_code>0
  EOT

  metric_descriptor {
    metric_kind = "DELTA"
    value_type  = "INT64"
    unit        = "1"
    labels {
      key         = "method"
      value_type  = "STRING"
    }
    labels {
      key         = "path"
      value_type  = "STRING"
    }
    labels {
      key         = "status"
      value_type  = "STRING"
    }
  }

  label_extractors = {
    "method" = 'EXTRACT(jsonPayload.method)'
    "path"   = 'EXTRACT(jsonPayload.route)'
    "status" = 'EXTRACT(jsonPayload.status_code)'
  }
}

# Log-based metric: Analysis job status
resource "google_logging_metric" "analysis_job_status" {
  name   = "eeg-lab-${var.environment}-analysis-job-status"
  description = "Analysis job status changes"
  filter = <<-EOT
    resource.type="cloud_run_revision"
    (resource.labels.service_name="eeg-lab-${var.environment}-api" OR resource.labels.service_name="eeg-lab-${var.environment}-worker")
    jsonPayload.action=~"analysis\\.(submitted|completed|failed|cancelled)"
  EOT

  metric_descriptor {
    metric_kind = "DELTA"
    value_type  = "INT64"
    unit        = "1"
    labels {
      key         = "action"
      value_type  = "STRING"
    }
    labels {
      key         = "status"
      value_type  = "STRING"
    }
    labels {
      key         = "error_code"
      value_type  = "STRING"
    }
  }

  label_extractors = {
    "action"      = 'EXTRACT(jsonPayload.action)'
    "status"      = 'EXTRACT(jsonPayload.metadata.status)'
    "error_code"  = 'EXTRACT(jsonPayload.metadata.error_code)'
  }
}

# Log-based metric: Outbox dispatch
resource "google_logging_metric" "outbox_dispatch" {
  name   = "eeg-lab-${var.environment}-outbox-dispatch"
  description = "Outbox dispatcher outcomes"
  filter = <<-EOT
    resource.type="cloud_run_revision"
    resource.labels.service_name="eeg-lab-${var.environment}-dispatcher"
    jsonPayload.claimed>0
  EOT

  metric_descriptor {
    metric_kind = "DELTA"
    value_type  = "INT64"
    unit        = "1"
    labels {
      key         = "outcome"
      value_type  = "STRING"
    }
  }

  label_extractors = {
    "outcome" = 'EXTRACT(jsonPayload.outcome)'
  }
}

# ============================================================
# Alert Policies
# ============================================================

# Alert: High API error rate
resource "google_monitoring_alert_policy" "api_high_error_rate" {
  display_name = "EEG Lab ${var.environment} - High API Error Rate"
  combiner     = "OR"
  conditions {
    display_name = "5xx error rate > 5% for 5 minutes"
    condition_threshold {
      filter = 'metric.type="logging.googleapis.com/user/eeg-lab-${var.environment}-http-request-count" AND resource.type="cloud_run_revision" AND resource.labels.service_name="eeg-lab-${var.environment}-api" AND metric.label.status=~"5.."'
      duration   = "300s"
      comparison = "COMPARISON_GT"
      threshold_value = 0.05
      trigger {
        count = 1
      }
      aggregations {
        alignment_period = "60s"
        per_series_aligner = "ALIGN_RATE"
        cross_series_reducer = "REDUCE_SUM"
        group_by_fields = ["metric.label.status"]
      }
      denominator_filter = 'metric.type="logging.googleapis.com/user/eeg-lab-${var.environment}-http-request-count" AND resource.type="cloud_run_revision" AND resource.labels.service_name="eeg-lab-${var.environment}-api"'
      denominator_aggregations {
        alignment_period = "60s"
        per_series_aligner = "ALIGN_RATE"
        cross_series_reducer = "REDUCE_SUM"
      }
    }
  }
  notification_channels = var.notification_channels
  alert_strategy {
    auto_close = "1800s"
  }
  labels = {
    environment = var.environment
    severity    = "critical"
  }
}

# Alert: API latency p99 > 2s
resource "google_monitoring_alert_policy" "api_high_latency" {
  display_name = "Eeg-lab ${var.environment} - High API Latency (p99)"
  combiner     = "OR"
  conditions {
    display_name = "p99 latency > 2s for 5 minutes"
    condition_threshold {
      filter = 'metric.type="logging.googleapis.com/user/eeg-lab-${var.environment}-http-request-duration" AND resource.type="cloud_run_revision" AND resource.labels.service_name="eeg-lab-${var.environment}-api"'
      duration   = "300s"
      comparison = "COMPARISON_GT"
      threshold_value = 2000
      trigger {
        count = 1
      }
      aggregations {
        alignment_period = "60s"
        per_series_aligner = "ALIGN_PERCENTILE_99"
        cross_series_reducer = "REDUCE_MEAN"
      }
    }
  }
  notification_channels = var.notification_channels
  labels = {
    environment = var.environment
    severity    = "warning"
  }
}

# Alert: Analysis job failure rate
resource "google_monitoring_alert_policy" "analysis_failure_rate" {
  display_name = "EEG Lab ${var.environment} - Analysis Job Failures"
  combiner     = "OR"
  conditions {
    display_name = "Failed jobs > 10% of completed in 15 minutes"
    condition_threshold {
      filter = 'metric.type="logging.googleapis.com/user/eeg-lab-${var.environment}-analysis-job-status" AND metric.label.action="analysis.failed"'
      duration   = "900s"
      comparison = "COMPARISON_GT"
      threshold_value = 3
      trigger {
        count = 1
      }
      aggregations {
        alignment_period = "300s"
        per_series_aligner = "ALIGN_RATE"
        cross_series_reducer = "REDUCE_SUM"
      }
    }
  }
  notification_channels = var.notification_channels
  labels = {
    environment = var.environment
    severity    = "warning"
  }
}

# Alert: Outbox dispatcher exhaustion
resource "google_monitoring_alert_policy" "outbox_exhausted" {
  display_name = "EEG Lab ${var.environment} - Outbox Events Exhausted"
  combiner     = "OR"
  conditions {
    display_name = "Outbox events exhausted"
    condition_threshold {
      filter = 'metric.type="logging.googleapis.com/user/eeg-lab-${var.environment}-outbox-dispatch" AND metric.label.outcome="exhausted"'
      duration   = "60s"
      comparison = "COMPARISON_GT"
      threshold_value = 0
      trigger {
        count = 1
      }
      aggregations {
        alignment_period = "60s"
        per_series_aligner = "ALIGN_SUM"
        cross_series_reducer = "REDUCE_SUM"
      }
    }
  }
  notification_channels = var.notification_channels
  labels = {
    environment = var.environment
    severity    = "critical"
  }
}

# Alert: Cloud SQL CPU high
resource "google_monitoring_alert_policy" "cloudsql_cpu_high" {
  display_name = "EEG Lab ${var.environment} - Cloud SQL CPU High"
  combiner     = "OR"
  conditions {
    display_name = "CPU utilization > 80% for 10 minutes"
    condition_threshold {
      filter = 'metric.type="cloudsql.googleapis.com/database/cpu/utilization" AND resource.type="cloudsql_database" AND resource.labels.database_id=~"eeg-lab-${var.environment}-db"'
      duration   = "600s"
      comparison = "COMPARISON_GT"
      threshold_value = 0.8
      trigger {
        count = 1
      }
      aggregations {
        alignment_period = "60s"
        per_series_aligner = "ALIGN_MEAN"
        cross_series_reducer = "REDUCE_MEAN"
      }
    }
  }
  notification_channels = var.notification_channels
  labels = {
    environment = var.environment
    severity    = "warning"
  }
}

# Alert: Cloud SQL storage near capacity
resource "google_monitoring_alert_policy" "cloudsql_storage_high" {
  display_name = "EEG Lab ${var.environment} - Cloud SQL Storage High"
  combiner     = "OR"
  conditions {
    display_name = "Storage utilization > 85%"
    condition_threshold {
      filter = 'metric.type="cloudsql.googleapis.com/database/disk/bytes_used" AND resource.type="cloudsql_database" AND resource.labels.database_id=~"eeg-lab-${var.environment}-db"'
      duration   = "3600s"
      comparison = "COMPARISON_GT"
      threshold_value = 0.85
      trigger {
        count = 1
      }
      aggregations {
        alignment_period = "3600s"
        per_series_aligner = "ALIGN_MEAN"
        cross_series_reducer = "REDUCE_MEAN"
        group_by_fields = ["resource.label.database_id"]
      }
      denominator_filter = 'metric.type="cloudsql.googleapis.com/database/disk/bytes_used" AND resource.type="cloudsql_database" AND resource.labels.database_id=~"eeg-lab-${var.environment}-db"'
      denominator_aggregations {
        alignment_period = "3600s"
        per_series_aligner = "ALIGN_MEAN"
        cross_series_reducer = "REDUCE_MEAN"
        group_by_fields = ["resource.label.database_id"]
      }
    }
  }
  notification_channels = var.notification_channels
  labels = {
    environment = var.environment
    severity    = "warning"
  }
}

# Alert: Redis memory high
resource "google_monitoring_alert_policy" "redis_memory_high" {
  display_name = "Eeg-lab ${var.environment} - Redis Memory High"
  combiner     = "OR"
  conditions {
    display_name = "Memory utilization > 85%"
    condition_threshold {
      filter = 'metric.type="redis.googleapis.com/memory/used_bytes" AND resource.type="redis_instance" AND resource.labels.instance_id=~"EEG Lab-${var.environment}-redis"'
      duration   = "300s"
      comparison = "COMPARISON_GT"
      threshold_value = 0.85
      trigger {
        count = 1
      }
      aggregations {
        alignment_period = "60s"
        per_series_aligner = "ALIGN_MEAN"
        cross_series_reducer = "REDUCE_MEAN"
        group_by_fields = ["resource.label.instance_id"]
      }
      denominator_filter = 'metric.type="redis.googleapis.com/memory/limit_bytes" AND resource.type="redis_instance" AND resource.labels.instance_id=~"eeg-lab-${var.environment}-redis"'
      denominator_aggregations {
        alignment_period = "60s"
        per_series_aligner = "ALIGN_MEAN"
        cross_series_reducer = "REDUCE_MEAN"
        group_by_fields = ["resource.label.instance_id"]
      }
    }
  }
  notification_channels = var.notification_channels
  labels = {
    environment = var.environment
    severity    = "warning"
  }
}

# ============================================================
# Uptime Check (API health)
# ============================================================

resource "google_monitoring_uptime_check_config" "api_health" {
  display_name = "EEG Lab ${var.environment} API Health Check"
  timeout      = "10s"
  period       = "60s"

  http_check {
    path        = "/health/ready"
    port        = 443
    use_ssl     = true
    validate_ssl = true
    request_method = "GET"
  }

  monitored_resource {
    type = "uptime_url"
    labels = {
      host = replace(google_cloud_run_v2_service.api.uri, "https://", "")
      project_id = var.project_id
    }
  }

  content_matchers {
    content = "healthy"
    matcher = "CONTAINS_STRING"
  }
}
