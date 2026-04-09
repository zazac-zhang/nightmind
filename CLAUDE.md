# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Common Development Commands

### Building and Running
- `make dev` or `cargo run` — Start the development server
- `make dev-watch` — Start with hot-reload (requires cargo-watch)
- `make build` — Build release binary
- `make check` — Run clippy and format checks

### Testing
- `make test` or `cargo test` — Run all tests
- `make test-coverage` — Generate coverage report (requires cargo-tarpaulin)
- `cargo test -- test_name` — Run a specific test

### Database
- `make db-migrate` — Run SQL migrations using sqlx-cli
- `make db-shell` — Connect to PostgreSQL shell
- `make db-reset` — Reset database (destructive: recreates from scratch)

### Docker Services
- `make docker-up` — Start PostgreSQL, Redis, Qdrant
- `make docker-down` — Stop all Docker services
- `make docker-logs` — View logs from all services

### Code Quality
- `make fmt` or `cargo fmt` — Format code
- `make lint` or `cargo clippy` — Run linter

### Quick Start
- `make quickstart` — Full setup: docker services + migrations + server

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    API Layer                        │
│  (handlers, websocket, router, middleware)         │
├─────────────────────────────────────────────────────┤
│                   Core Layer                        │
│     (agent, session management, content)            │
├─────────────────────────────────────────────────────┤
│                 Repository Layer                    │
│      (database models, postgres, redis)             │
├─────────────────────────────────────────────────────┤
│                 Services Layer                      │
│     (audio, STT, TTS, integration, vector)          │
├─────────────────────────────────────────────────────┤
│                 Infrastructure                      │
│        (config, error, logging)                     │
└─────────────────────────────────────────────────────┘
```

### Core Modules

| Module | Purpose |
|--------|---------|
| `src/core/agent/` | Rig Agent integration, prompts |
| `src/core/session/` | SessionManager, state machine, topic stack |
| `src/core/content/` | Content transformation (code→metaphor) |
| `src/api/` | Axum router, WebSocket, REST handlers |
| `src/repository/` | PostgreSQL models and queries |
| `src/services/` | STT, TTS, vector store, integrations |

### Data Flow

1. WebSocket connection → `src/api/handlers/websocket.rs`
2. SessionManager tracks active sessions
3. NightMindAgent processes messages with Rig
4. PostgreSQL for persistence, Redis for caching

### Services

- **PostgreSQL** (localhost:5432) — Primary database with pgvector
- **Redis** (localhost:6379) — Session caching
- **Qdrant** (localhost:6333) — Vector database

## Testing

- Unit tests: Inline in source files (`#[cfg(test)]` modules)
- Integration tests: `tests/integration_tests.rs`
- WebSocket tests: `tests/websocket_tests.rs`

## Configuration

Configuration is loaded from environment variables via `src/config/settings.rs`:
- `Settings::load()` reads from environment
- Key sections: `server`, `database`, `redis`, `ai`
- Requires `OPENAI_API_KEY` for LLM operations

## Error Handling

- Custom error type: `NightMindError` in `src/error.rs`
- Result type alias: `Result<T> = std::result::Result<T, NightMindError>`
- Errors propagate with `?` operator

## Adding New Features

1. **New API endpoint**: Add handler in `src/api/handlers/`, register in `src/api/router.rs`
2. **New database model**: Add to `src/repository/models/`, create repository queries
3. **New agent capability**: Extend `src/core/agent/builder.rs`

## Documentation

- [README](README.md) — Quick start and overview
- [Architecture](doc/architecture.md) — Detailed architecture
- [API Design](doc/api.md) — WebSocket and REST API
- [Development Plan](docs/plan.md) — Current tasks and roadmap

## Migration System

Migrations in `migrations/` directory use sqlx:
- `xxx.up.sql` — Apply migration
- `xxx.down.sql` — Rollback migration
- Run with `make db-migrate` or `sqlx migrate run`
