# Filebrowser

A file browser built with Leptos (SSR + Hydration) and Axum.

## Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) (for dev server with hot reload)

```bash
cargo install cargo-leptos
```

## Project Structure

| Directory | Crate | Role |
|-----------|-------|------|
| `app/` | `filebrowser-app` | Server startup (Axum + Leptos SSR) |
| `frontend/` | `filebrowser-frontend` | UI components, Router, hydration entry |
| `backend/` | `filebrowser-backend` | API endpoints, server functions |
| `types/` | `filebrowser-types` | Shared types |

## Development

Start the dev server with hot reload:

```bash
cargo leptos watch
```

The app will be available at `http://127.0.0.1:3000`.

## Checks

Run all checks locally before pushing:

```bash
# Format
cargo fmt --all -- --check

# Lint (SSR side)
cargo clippy -p filebrowser-app --features ssr

# Lint (hydrate/WASM side)
cargo clippy -p filebrowser-frontend --features hydrate

# Tests (starts the server and tests API endpoints)
cargo test -p filebrowser-app --features ssr

# All backend/types tests
cargo test -p filebrowser-backend --features ssr
cargo test -p filebrowser-types
```

Or run everything at once:

```bash
cargo fmt --all -- --check \
  && cargo clippy -p filebrowser-app --features ssr \
  && cargo clippy -p filebrowser-frontend --features hydrate \
  && cargo test -p filebrowser-app --features ssr \
  && cargo test -p filebrowser-backend --features ssr \
  && cargo test -p filebrowser-types
```

## CI

GitHub Actions runs the following on every push and PR to `main`:

- **Format** — `cargo fmt --check`
- **Clippy** — lint both SSR and hydrate builds
- **Test** — integration tests that start the server and verify API endpoints
- **Build** — full SSR build
