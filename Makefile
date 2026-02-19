.PHONY: help up down dev-backend dev-frontend build logs clean test

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# ─── Docker Compose ──────────────────────────────────────────

up: ## Start all services (production mode)
	docker compose up --build -d
	@echo ""
	@echo "Services running:"
	@echo "  Frontend:  http://localhost:3000"
	@echo "  Backend:   http://localhost:8080"
	@echo "  Postgres:  localhost:5432"
	@echo "  Redis:     localhost:6379"

down: ## Stop all services
	docker compose down

down-clean: ## Stop services and remove volumes
	docker compose down -v

# ─── Development ─────────────────────────────────────────────

dev-backend: ## Run Go backend in dev mode (requires local Go)
	cd backend && go run ./cmd/server

dev-frontend: ## Run Vite dev server (requires local Node)
	cd frontend && npm run dev

dev-deps: ## Start only Postgres + Redis for local development
	docker compose up -d postgres redis

# ─── Build & Test ────────────────────────────────────────────

build: ## Build all Docker images
	docker compose build

test-backend: ## Run Go tests
	cd backend && go test -v ./...

# ─── Utilities ───────────────────────────────────────────────

logs: ## Tail logs from all services
	docker compose logs -f --tail=50

status: ## Show service status
	docker compose ps

clean: down-clean ## Full cleanup
	@echo "Cleaned up!"
