output "alert_policy_names" {
  description = "Names of created alert policies"
  value       = [
    google_monitoring_alert_policy.api_high_error_rate.name,
    google_monitoring_alert_policy.api_high_latency.name,
    google_monitoring_alert_policy.analysis_failure_rate.name,
    google_monitoring_alert_policy.outbox_exhausted.name,
    google_monitoring_alert_policy.cloudsql_cpu_high.name,
    google_monitoring_alert_policy.cloudsql_storage_high.name,
    google_monitoring_alert_policy.redis_memory_high.name,
  ]
}

output "uptime_check_name" {
  description = "Uptime check name"
  value       = google_monitoring_uptime_check_config.api_health.name
}

output "log_metric_names" {
  description = "Names of created log-based metrics"
  value       = [
    google_logging_metric.http_request_duration.name,
    google_logging_metric.http_request_count.name,
    google_logging_metric.analysis_job_status.name,
    google_logging_metric.outbox_dispatch.name,
  ]
}