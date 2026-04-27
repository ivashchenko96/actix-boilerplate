# Actix Boilerplate

Production-grade, modular **Actix Web** backend template for SaaS and enterprise systems.

This repository provides:
- structured modules (`health`, `auth`, `users`)
- shared application context and service container
- PostgreSQL, Redis, NATS, S3, email integrations
- cron scheduler support
- OpenAPI + Swagger UI
- i18n foundation
- deployment and CI/CD starter assets

---

## Table of Contents
- [Why this boilerplate](#why-this-boilerplate)
- [Current architecture](#current-architecture)
- [Quick start](#quick-start)
- [Configuration](#configuration)
- [Developer workflows](#developer-workflows)
- [Operations](#operations)
- [Documentation map](#documentation-map)
- [AI agent docs](#ai-agent-docs)

---

## Why this boilerplate

Use this project when you need an API backend that is:
- opinionated but extensible
- ready for layered enterprise concerns (security, observability, deployment)
- easy to scale by module and by service boundaries

---

## Current architecture

At startup, the app:
1. loads env and settings
2. initializes logging
3. builds `AppContext` (DB + external services + i18n + feature flags)
4. creates HTTP server with middleware stack
5. registers modules and routes
6. starts optional cron scheduler and optional metrics endpoint

High-level flow:
- `src/main.rs` → bootstrapping
- `src/app.rs` → Actix app assembly
- `src/context.rs` → runtime service wiring
- `src/modules/*` → domain modules
- `src/middleware/*` → cross-cutting concerns

For deeper details see [`ARCHITECTURE.md`](/home/runner/work/actix-boilerplate/actix-boilerplate/ARCHITECTURE.md).

---

## Quick start

### 1) Prerequisites
- Rust stable toolchain
- PostgreSQL
- Redis
- NATS
- (optional) Docker / Docker Compose

### 2) Environment
```bash
cp .env.example .env
# edit .env with your secrets and service URLs
```

### 3) Run locally
```bash
make check
make run
```

Or with watcher:
```bash
make dev
```

### 4) Run tests
```bash
make test
```

---

## Configuration

Config sources:
1. `config/default.toml`
2. `config/{ENVIRONMENT}.toml`
3. `config/local.toml` (optional local override)
4. `APP__...` environment overrides

Core envs commonly used:
- `ENVIRONMENT`
- `LOG_LEVEL`
- `DATABASE_URL`
- `REDIS_URL`
- `NATS_URL`
- `JWT_SECRET`
- `SMTP_*`

More details: [`docs/configuration.md`](/home/runner/work/actix-boilerplate/actix-boilerplate/docs/configuration.md).

---

## Developer workflows

### Build and quality
```bash
make build
make check
make fmt
make clippy
```

### Database migrations
```bash
make migrate
make rollback
```

### Docker
```bash
make docker-build
make docker-compose-up
```

---

## Operations

- Health endpoints live under module routes (`/health` scope).
- Swagger UI available at `/swagger-ui/`.
- Metrics endpoint (`/metrics`) can be toggled via feature flags.
- Cron jobs can be enabled/disabled in config.

Deployment references are in `/deploy` and [`docs/deployment.md`](/home/runner/work/actix-boilerplate/actix-boilerplate/docs/deployment.md).

---

## Documentation map

- Architecture: [`ARCHITECTURE.md`](/home/runner/work/actix-boilerplate/actix-boilerplate/ARCHITECTURE.md)
- Modules: [`docs/modules.md`](/home/runner/work/actix-boilerplate/actix-boilerplate/docs/modules.md)
- i18n: [`docs/i18n.md`](/home/runner/work/actix-boilerplate/actix-boilerplate/docs/i18n.md)
- Logging: [`docs/logging.md`](/home/runner/work/actix-boilerplate/actix-boilerplate/docs/logging.md)
- Cron: [`docs/cron.md`](/home/runner/work/actix-boilerplate/actix-boilerplate/docs/cron.md)

---

## AI agent docs

- Agent behavior and contribution skill model: [`SKILLS.md`](/home/runner/work/actix-boilerplate/actix-boilerplate/SKILLS.md)
- MCP integration and safety contract: [`AI_MCP.md`](/home/runner/work/actix-boilerplate/actix-boilerplate/AI_MCP.md)

These docs are intended for autonomous/semi-autonomous coding agents and reviewers.
