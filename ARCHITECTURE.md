# ARCHITECTURE

Enterprise-oriented architecture overview for `actix-boilerplate`.

## 1. Design goals

- Strong modularity by domain
- Explicit cross-cutting middleware
- Clear runtime dependency graph
- Operational readiness (health, logs, metrics, docs, deploy)
- AI-assisted development compatibility

## 2. Layered model

### Entry and bootstrap
- `src/main.rs`
  - loads env
  - configures tracing
  - builds `AppContext`
  - starts HTTP server

### App composition
- `src/app.rs`
  - middleware stack assembly
  - module registration
  - Swagger and metrics setup
  - optional cron boot

### Runtime context
- `src/context.rs`
  - settings
  - PostgreSQL pool
  - Redis/NATS/Typesense/Storage/Email services
  - i18n and feature flags

### Domain modules
- `src/modules/*`
  - route/controller/service/repository/dto/models/errors
  - registered via `ModuleRegistry`

### Shared concerns
- `src/middleware/*` for auth, locale, request-id, logging, security, rate limit
- `src/errors.rs` for unified API + error response contract
- `src/utils/*` for reusable infrastructure-level helpers

## 3. Request flow

1. Request enters Actix app
2. Middlewares enrich/guard request
3. Route dispatch to module controller
4. Controller delegates to service layer
5. Service uses repository and shared services
6. Response returned through middleware chain
7. Structured logs emitted

## 4. Persistence and integration boundaries

- Primary DB: PostgreSQL (`sqlx`)
- Cache/session/rate state helpers: Redis (`fred`)
- Event messaging: NATS
- Object storage: S3-compatible through AWS SDK
- Email transport: SMTP (`lettre`)

Each integration is wrapped in a service module to isolate vendor details.

## 5. Module contract

`AppModule` currently defines:
- `name`
- `register_routes`
- `register_jobs`
- `register_permissions`
- `register_openapi`

This keeps module growth predictable and enables future plugin-like module additions.

## 6. Operational architecture

- `/swagger-ui/*` for API docs
- `/metrics` for Prometheus (feature-flag controlled)
- health checks for service readiness
- cron scheduler for periodic jobs
- deployment presets under `deploy/`
- CI/CD workflows under `.github/workflows/`

## 7. Security posture (baseline)

- JWT-based auth middleware
- request identity propagation (`X-Request-ID`)
- security headers middleware
- CORS middleware
- password hashing helpers
- centralized error handling and minimal info leakage

## 8. Extending architecture

Recommended path for new domain:
1. add `src/modules/<domain>/`
2. implement route/controller/service/repository + dto/models/errors
3. register module in `src/app.rs`
4. add DB migration(s)
5. add tests and docs

## 9. Known maturity notes

This repo is a strong scaffold and still expects product-specific hardening:
- stricter auth/token lifecycle rules
- deeper observability dashboards/alerts
- domain-specific validation/authorization rules
- production secrets and infra policy integration

