# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Filebrowser is a Rust workspace project with three crates:

- **backend** (`filebrowser-backend`) — Async server using Tokio
- **frontend** (`filebrowser-frontend`) — Client/UI crate
- **types** (`filebrowser-types`) — Shared types between backend and frontend

## Build Commands

```bash
# Build entire workspace
cargo build

# Build a single crate
cargo build -p filebrowser-backend

# Run a specific crate
cargo run -p filebrowser-backend

# Run tests (all crates)
cargo test

# Run tests for a single crate
cargo test -p filebrowser-types

# Run a single test by name
cargo test -p filebrowser-backend test_name

# Check without building
cargo check

# Lint
cargo clippy --workspace

# Format
cargo fmt --all
```

## Architecture

Rust 2024 edition workspace using resolver v3. The `types` crate is intended as the shared dependency between `backend` and `frontend`, keeping type definitions in one place.
