# Makefile for actix-boilerplate

.PHONY: help build test run clean dev setup install migrate rollback check fmt clippy audit doc docker-build docker-run deploy

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

install: ## Install dependencies
	cargo install sqlx-cli --features postgres
	cargo install cargo-watch
	cargo install cargo-audit

setup: install ## Setup development environment
	cp .env.example .env
	@echo "Please configure your .env file with appropriate values"

build: ## Build the application
	cargo build

build-release: ## Build for production
	cargo build --release

test: ## Run tests
	cargo test

test-watch: ## Run tests in watch mode
	cargo watch -x test

run: ## Run the application
	cargo run

dev: ## Run in development mode with auto-reload
	cargo watch -x run

check: ## Check code without building
	cargo check

fmt: ## Format code
	cargo fmt

clippy: ## Run clippy lints
	cargo clippy -- -D warnings

audit: ## Security audit
	cargo audit

doc: ## Generate documentation
	cargo doc --open

clean: ## Clean build artifacts
	cargo clean

migrate: ## Run database migrations
	sqlx migrate run

rollback: ## Rollback last migration
	sqlx migrate revert

migrate-reset: ## Reset database and re-run migrations
	sqlx database drop -y
	sqlx database create
	sqlx migrate run

docker-build: ## Build Docker image
	docker build -t actix-boilerplate .

docker-run: ## Run Docker container
	docker run -p 8000:8000 actix-boilerplate

docker-compose-up: ## Start services with docker-compose
	docker-compose up -d

docker-compose-down: ## Stop services with docker-compose
	docker-compose down

docker-compose-prod: ## Start production services
	docker-compose -f docker-compose.prod.yml up -d

deploy: ## Deploy to production
	./scripts/deploy.sh