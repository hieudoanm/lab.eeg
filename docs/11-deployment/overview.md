# EEG Lab — Deployment and Operations

## 1. Purpose

This document defines how to run, test, deploy, monitor, maintain, and safely tear down EEG Lab.

EEG Lab is an independent research-oriented neuroscience SaaS portfolio project built with:

- Next.js, React, and TypeScript.
- Python, FastAPI, and Pydantic.
- PostgreSQL and Psycopg 3.
- Celery and Redis.
- NumPy and Plotly.js.
- Private Google Cloud Storage.
- Google Cloud Platform.
- Terraform.
- GitHub Actions.

The deployment strategy prioritises reproducibility, security, reasonable operating costs, and a clear path from local development to a publicly accessible demonstration.

The exact cloud configuration must be verified against current provider capabilities and project requirements before implementation.

