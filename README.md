# Fitness Assistant AI

A privacy-focused, self-hosted health and longevity platform built with Rust.

## Features

- Weight and body composition tracking
- Nutrition and food logging
- Exercise and workout tracking
- Hydration monitoring
- Sleep analysis
- Heart rate and HRV monitoring
- AI-powered health insights
- External device integrations (Apple Health, Garmin, Oura, Whoop)

## Architecture

The project uses a Cargo workspace with three crates:

- `backend/` - Axum-based REST API server
- `shared/` - Shared types, models, and validation
- `wasm/` - WebAssembly modules for browser calculations

## Prerequisites

- Rust 1.75+
- PostgreSQL 15+
- Redis 7+
- Docker (for development)

## Development Setup

1. Start the development services:
   ```bash
   docker-compose -f docker-compose.dev.yml up -d
   ```

2. Run database migrations:
   ```bash
   sqlx migrate run --source backend/migrations
   ```

3. Start the backend server:
   ```bash
   cargo run --bin fitness-assistant-backend
   ```

## Configuration

Configuration is loaded hierarchically:
1. Default values (in code)
2. TOML config files (`config/development.toml`, `config/production.toml`)
3. Environment variables (prefix: `FA_`)

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run property-based tests
cargo test --features proptest
```

## License

MIT
