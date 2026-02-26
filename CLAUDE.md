# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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
- Integration tests are in `tests/integration_tests.rs` and `tests/websocket_tests.rs`

### Database
- `make db-migrate` — Run SQL migrations using sqlx-cli
- `make db-shell` — Connect to PostgreSQL shell
- `make db-reset` — Reset database (destructive: recreates from scratch)
- `make db-backup` — Backup database to `backups/`
- `make db-restore FILE=backups/xxx.sql.gz` — Restore from backup

### Docker Services
- `make docker-up` — Start PostgreSQL, Redis, Qdrant
- `make docker-down` — Stop all Docker services
- `make tools-up` — Start pgAdmin (port 5050) and Redis Commander (port 8081)
- `make docker-logs` — View logs from all services

### Code Quality
- `make fmt` or `cargo fmt` — Format code
- `make lint` or `cargo clippy` — Run linter
- `make docs` or `cargo doc --open` — Generate and open documentation

### Quick Start
- `make quickstart` — Full setup: docker services + migrations + server

## Architecture Overview

NightMind is a Rust-based AI learning companion application with a layered architecture:

```
┌─────────────────────────────────────────────────────┐
│                    API Layer                        │
│  (handlers, websocket, router, dto, middleware)    │
├─────────────────────────────────────────────────────┤
│                   Core Layer                        │
│     (agent system, session management, content)     │
├─────────────────────────────────────────────────────┤
│                 Repository Layer                    │
│      (database models, user, session, knowledge)    │
├─────────────────────────────────────────────────────┤
│                 Services Layer                      │
│     (audio, STT, TTS, integration, vector)         │
├─────────────────────────────────────────────────────┤
│                 Infrastructure                      │
│        (config, error, logging)                    │
└─────────────────────────────────────────────────────┘
```

### Core Modules

**Agent System** (`src/core/agent/`):
- `builder.rs` — AgentBuilder for constructing AI agents with fluent API
- `prompts.rs` — Prompt templates and personality configuration
- `tools.rs` — Agent tools (currently placeholder for rig-core integration)

**Session Management** (`src/core/session/`):
- `manager.rs` — SessionManager tracks active sessions in-memory
- `state.rs` — Session state models (Warmup, DeepDive, Review, Seed, Closing)
- `snapshot.rs` — Session snapshot/restore functionality
- `topic_stack.rs` — Topic stack for conversation tracking

**Content Processing** (`src/core/content/`):
- `transformer.rs` — Content transformation logic
- `rhythm.rs` — Pacing and fatigue detection for bedtime sessions

### Data Flow

1. **WebSocket Connection** (`src/api/websocket.rs`) — Main entry point for real-time interaction
2. **Session Manager** — Tracks active sessions, handles cleanup of expired sessions
3. **Agent System** — Manages AI agent instances with per-session configuration
4. **Repository Layer** — PostgreSQL for persistent data, Redis for caching, Qdrant for vectors

### Key Architectural Decisions

- **Axum + Tower** — Web framework with middleware for CORS, tracing, compression, timeout
- **Rig Agent Framework** (rig-core) — AI agent orchestration (integration in progress)
- **Session State Machine** — Sessions progress through states: Warmup → DeepDive → Review → Seed → Closing
- **Repository Pattern** — Abstract data access with PgPool for PostgreSQL, RedisManager for caching
- **Async/Await** — Tokio runtime throughout, with RwLock for concurrent session access

### Services

- **PostgreSQL** (localhost:5432) — Primary database with pgvector for vector operations
- **Redis** (localhost:6379) — Session caching and transient state
- **Qdrant** (localhost:6333) — Vector database for semantic search
- **pgAdmin** (localhost:5050) — PostgreSQL admin UI (requires `make tools-up`)
- **Redis Commander** (localhost:8081) — Redis admin UI (requires `make tools-up`)

### Testing Setup

- Unit tests: Inline in source files (module-level `#[cfg(test)]` modules)
- Integration tests: `tests/integration_tests.rs` — Uses TestServer for full HTTP stack testing
- WebSocket tests: `tests/websocket_tests.rs`
- Test utilities: `tokio-test`, `mockall`, `wiremock`, `httpmock`, `pretty_assertions`

### Configuration

Configuration is loaded from environment variables via `src/config/settings.rs`:
- `Settings::load()` reads from environment (see `.env.example` for structure)
- Key config sections: `server`, `database`, `redis`, `ai`, `logging`
- AI config requires `OPENAI_API_KEY` for LLM operations

### Error Handling

- Custom error type: `NightMindError` in `src/error.rs`
- Result type alias: `Result<T> = std::result::Result<T, NightMindError>`
- Errors are propagated with `?` operator and converted at layer boundaries

### Adding New Features

1. **New API endpoint**: Add handler in `src/api/handlers.rs`, register in `src/api/router.rs`
2. **New database model**: Add to `src/repository/models/`, create repository in `src/repository/`
3. **New agent capability**: Extend `src/core/agent/builder.rs` or `src/core/agent/tools.rs`
4. **New session state**: Add to `SessionState` enum in `src/repository/models/session.rs`

### Migration System

Migrations in `migrations/` directory use sqlx:
- `xxx.up.sql` — Apply migration
- `xxx.down.sql` — Rollback migration
- Run with `make db-migrate` or `sqlx migrate run`
- Rollback with `make db-migrate-rollback` or `sqlx migrate revert`
