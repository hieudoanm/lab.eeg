"""Prometheus metrics for API and worker observability."""

from __future__ import annotations

import time
from collections.abc import AsyncIterator, Callable
from contextlib import asynccontextmanager
from typing import Any

from fastapi import FastAPI, Request
from prometheus_client import CONTENT_TYPE_LATEST, Counter, Gauge, Histogram, generate_latest
from starlette.responses import Response

# ---------------------------------------------------------------------------
# API metrics
# ---------------------------------------------------------------------------

HTTP_REQUESTS_TOTAL = Counter(
    "eeg-lab_http_requests_total",
    "Total number of HTTP requests by method, path, and status.",
    ["method", "path", "status"],
)

HTTP_REQUEST_DURATION_SECONDS = Histogram(
    "eeg-lab_http_request_duration_seconds",
    "HTTP request latency in seconds.",
    ["method", "path"],
    buckets=[0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
)

HTTP_REQUESTS_IN_PROGRESS = Gauge(
    "eeg-lab_http_requests_in_progress",
    "Number of HTTP requests currently being processed.",
    ["method", "path"],
)

# ---------------------------------------------------------------------------
# Job metrics
# ---------------------------------------------------------------------------

ANALYSIS_JOBS_TOTAL = Counter(
    "eeg-lab_analysis_jobs_total",
    "Total number of analysis jobs by status.",
    ["status"],
)

ANALYSIS_JOB_DURATION_SECONDS = Histogram(
    "eeg-lab_analysis_job_duration_seconds",
    "Analysis job execution duration in seconds.",
    ["analysis_type", "status"],
    buckets=[1.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0],
)

ANALYSIS_JOB_RETRIES_TOTAL = Counter(
    "eeg-lab_analysis_job_retries_total",
    "Total number of analysis job retries.",
    ["analysis_type", "error_code"],
)

# ---------------------------------------------------------------------------
# Outbox metrics
# ---------------------------------------------------------------------------

OUTBOX_EVENTS_TOTAL = Counter(
    "eeg-lab_outbox_events_total",
    "Total number of outbox events by outcome.",
    ["event_type", "outcome"],
)

OUTBOX_DISPATCH_DURATION_SECONDS = Histogram(
    "eeg-lab_outbox_dispatch_duration_seconds",
    "Outbox dispatch batch duration in seconds.",
    buckets=[0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0],
)

# ---------------------------------------------------------------------------
# Storage metrics
# ---------------------------------------------------------------------------

STORAGE_OPERATIONS_TOTAL = Counter(
    "eeg-lab_storage_operations_total",
    "Total number of storage operations by operation and outcome.",
    ["operation", "outcome"],
)

STORAGE_OPERATION_DURATION_SECONDS = Histogram(
    "eeg-lab_storage_operation_duration_seconds",
    "Storage operation latency in seconds.",
    ["operation"],
    buckets=[0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0],
)

# ---------------------------------------------------------------------------
# Database metrics
# ---------------------------------------------------------------------------

DATABASE_OPERATIONS_TOTAL = Counter(
    "eeg-lab_database_operations_total",
    "Total number of database operations by operation and outcome.",
    ["operation", "outcome"],
)

DATABASE_OPERATION_DURATION_SECONDS = Histogram(
    "eeg-lab_database_operation_duration_seconds",
    "Database operation latency in seconds.",
    ["operation"],
    buckets=[0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0],
)

DATABASE_POOL_CONNECTIONS = Gauge(
    "eeg-lab_database_pool_connections",
    "Number of database pool connections by state.",
    ["state"],
)

# ---------------------------------------------------------------------------
# Worker metrics
# ---------------------------------------------------------------------------

WORKER_TASKS_TOTAL = Counter(
    "eeg-lab_worker_tasks_total",
    "Total number of worker tasks by task name and outcome.",
    ["task_name", "outcome"],
)

WORKER_TASK_DURATION_SECONDS = Histogram(
    "eeg-lab_worker_task_duration_seconds",
    "Worker task execution duration in seconds.",
    ["task_name"],
    buckets=[1.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0],
)


# ---------------------------------------------------------------------------
# Middleware helper
# ---------------------------------------------------------------------------


async def prometheus_middleware(request: Request, call_next: Callable[[Request], Any]) -> Response:
    """Middleware to record HTTP request metrics."""
    from starlette.responses import Response as StarletteResponse

    method = request.method
    path = request.scope.get("route", {}).get("path", request.url.path)

    HTTP_REQUESTS_IN_PROGRESS.labels(method=method, path=path).inc()
    start = time.perf_counter()
    try:
        response: StarletteResponse = await call_next(request)
        status = str(response.status_code)
    except Exception:
        status = "500"
        raise
    finally:
        duration = time.perf_counter() - start
        HTTP_REQUESTS_IN_PROGRESS.labels(method=method, path=path).dec()
        HTTP_REQUESTS_TOTAL.labels(method=method, path=path, status=status).inc()
        HTTP_REQUEST_DURATION_SECONDS.labels(method=method, path=path).observe(duration)
    return response


# ---------------------------------------------------------------------------
# Metrics endpoint
# ---------------------------------------------------------------------------


async def metrics_endpoint(request: Request) -> Response:
    """Expose Prometheus metrics at /metrics."""
    return Response(content=generate_latest(), media_type=CONTENT_TYPE_LATEST)


# ---------------------------------------------------------------------------
# Lifespan integration
# ---------------------------------------------------------------------------


@asynccontextmanager
async def metrics_lifespan(app: FastAPI) -> AsyncIterator[None]:
    """Lifespan context manager for metrics (no-op, here for symmetry)."""
    yield


def install_metrics(app: FastAPI) -> None:
    """Install Prometheus metrics middleware and endpoint."""
    app.middleware("http")(prometheus_middleware)
    app.add_route("/metrics", metrics_endpoint, methods=["GET"])
