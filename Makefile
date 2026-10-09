SHELL := /bin/bash
.DEFAULT_GOAL := help

PY := uv --directory packages/backend
WEB_DIR := packages/frontend

.PHONY: help install infra-up infra-down docker-up docker-down db-migrate db-reset \
        api-run worker dispatch-outbox dev dev-stop lint format typecheck test test-api test-web test-web-e2e \
        sample-recording cleanup-uploads check clean

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'

install: ## Install backend and frontend dependencies
	$(PY) sync --all-groups
	@if command -v pnpm >/dev/null 2>&1 && [ -f $(WEB_DIR)/package.json ]; then \
		pnpm install --frozen-lockfile || pnpm install; \
	else \
		echo "frontend not implemented yet; skipping pnpm install"; \
	fi

infra-up: ## Start PostgreSQL and Redis
	docker compose up -d postgres redis

infra-down: ## Stop PostgreSQL and Redis
	docker compose down

docker-up: ## Build and start the full application with Docker Compose
	docker compose up --build -d

docker-down: ## Stop Docker Compose services, preserving local volumes
	docker compose down

dev: ## Start everything locally (infra + migrations + API + worker + dispatcher + frontend)
	@echo "Starting EEG Lab locally..."
	$(MAKE) infra-up
	@echo "Waiting for databases..."
	@sleep 3
	$(MAKE) db-migrate
	@echo "Starting services in background..."
	@mkdir -p .local-data/uploads
	@$(PY) run uvicorn app.main:app --reload --app-dir packages/backend > .local-data/api.log 2>&1 & echo $$! > .local-data/api.pid
	@$(PY) run celery -A app.workers.celery_app.celery_app worker --loglevel=info > .local-data/worker.log 2>&1 & echo $$! > .local-data/worker.pid
	@while true; do $(PY) run python -m app.workers.outbox_dispatcher; sleep 5; done > .local-data/dispatcher.log 2>&1 & echo $$! > .local-data/dispatcher.pid
	@cd $(WEB_DIR) && pnpm dev > ../.local-data/frontend.log 2>&1 & echo $$! > .local-data/frontend.pid
	@echo ""
	@echo "EEG Lab started!"
	@echo "  Frontend: http://localhost:3000"
	@echo "  API:      http://localhost:8000"
	@echo "  API Docs: http://localhost:8000/docs"
	@echo ""
	@echo "Logs in .local-data/ (api.log, worker.log, dispatcher.log, frontend.log)"
	@echo "Run 'make dev-stop' to stop all services"

dev-stop: ## Stop all locally running services
	@echo "Stopping EEG Lab services..."
	@for pidfile in .local-data/*.pid; do \
		if [ -f "$$pidfile" ]; then \
			kill $$(cat "$$pidfile") 2>/dev/null || true; \
			rm "$$pidfile"; \
		fi; \
	done
	$(MAKE) infra-down
	@echo "All services stopped"

db-migrate: ## Apply pending SQL migrations
	$(PY) run python -m app.db.migrations upgrade

db-reset: ## Recreate the local database from migrations
	docker compose exec postgres psql -U eeg-lab -d postgres -c "DROP DATABASE IF EXISTS eeg-lab;" || true
	docker compose exec postgres psql -U eeg-lab -d postgres -c "CREATE DATABASE eeg-lab;" || true
	$(PY) run python -m app.db.migrations upgrade

api-run: ## Run the FastAPI application
	$(PY) run uvicorn app.main:app --reload --app-dir packages/backend

worker: ## Run the Celery analysis worker
	$(PY) run celery -A app.workers.celery_app.celery_app worker --loglevel=info

dispatch-outbox: ## Recover stale jobs and dispatch pending outbox events
	./scripts/dispatch_outbox.sh

sample-recording: ## Generate a synthetic .npy recording (VAR="--samples 4000")
	./scripts/generate_recording.sh $(VAR)

cleanup-uploads: ## Expire abandoned upload sessions and delete their objects
	./scripts/cleanup_uploads.sh

lint: ## Run formatting, lint, and type checks
	$(PY) run ruff format --check .
	$(PY) run ruff check .
	$(PY) run python -m mypy .
	@if command -v pnpm >/dev/null 2>&1 && [ -f $(WEB_DIR)/package.json ]; then \
		pnpm --dir $(WEB_DIR) lint && \
		pnpm --dir $(WEB_DIR) format:check; \
	fi

format: ## Apply formatting
	$(PY) run ruff format .
	$(PY) run ruff check --fix .
	@if command -v pnpm >/dev/null 2>&1 && [ -f $(WEB_DIR)/package.json ]; then \
		pnpm --dir $(WEB_DIR) format; \
	fi

typecheck: ## Run type checkers
	$(PY) run python -m mypy .
	@if command -v pnpm >/dev/null 2>&1 && [ -f $(WEB_DIR)/package.json ]; then \
		pnpm --dir $(WEB_DIR) typecheck; \
	fi

test-api: ## Run backend test suites with coverage enforcement
	$(PY) run pytest --cov=app --cov-report=term-missing

test-web: ## Run frontend unit tests
	@if [ -f $(WEB_DIR)/package.json ]; then pnpm --dir $(WEB_DIR) test; else echo "frontend not implemented yet"; fi

test-web-e2e: ## Run frontend Playwright smoke tests (requires browsers installed)
	@if [ -f $(WEB_DIR)/package.json ]; then pnpm --dir $(WEB_DIR) test:e2e; else echo "frontend not implemented yet"; fi

test: test-api test-web ## Run all test suites

check: lint typecheck test ## Run every local quality gate

clean: ## Remove local build and cache artefacts
	rm -rf .local-data packages/backend/.pytest_cache packages/backend/.mypy_cache packages/backend/.ruff_cache
	find . -name '__pycache__' -type d -prune -exec rm -rf {} +