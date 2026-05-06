# Foxide

**Foxide** is a self-hosted file browser written in Rust. The name combines **file** + **oxide** — a nod to the Rust ecosystem ("oxide" being the family of words around Rust/oxidation) wrapped around a "fox-ish" file-handling identity.

The frontend is built with [Yew](https://yew.rs) (struct-based components, CSR via [Trunk](https://trunkrs.dev)) and the backend with Axum. A single Axum server hosts the WebSocket / API endpoints and serves the static SPA bundle.

[日本語の README はこちら](./README_ja.md)

## Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`
- The standalone Tailwind CLI binary at `frontend/tailwindcss` and the daisyUI plugins (`frontend/daisyui.mjs`, `frontend/daisyui-theme.mjs`). Trunk's `pre_build` hook in `frontend/Trunk.toml` runs `./tailwindcss -i input.css -o output.css` automatically.

## Project Structure

| Directory   | Crate              | Role                                                                |
| ----------- | ------------------ | ------------------------------------------------------------------- |
| `app/`      | `foxide-app`       | Server startup (Axum, serves WS / API / static SPA bundle)          |
| `frontend/` | `foxide-frontend`  | Yew (struct-based) UI components and router; entry for `trunk`      |
| `backend/`  | `foxide-backend`   | API endpoints, WebSocket handler, persistence                       |
| `types/`    | `foxide-types`     | Shared types                                                        |

The Yew `App` component (top of the tree, `frontend/src/lib.rs`) opens the WebSocket connection in `Component::rendered(first_render=true)` and forwards messages to descendant pages via `ContextProvider<AppCtx>`.

## Development

```bash
# 1. Fetch the Tailwind CLI and daisyUI plugins (first time only)
cd frontend
curl -sLo tailwindcss https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64
curl -sLO https://github.com/saadeghi/daisyui/releases/latest/download/daisyui.mjs
curl -sLO https://github.com/saadeghi/daisyui/releases/latest/download/daisyui-theme.mjs
chmod +x tailwindcss
cd ..

# 2. Build the SPA bundle. trunk writes to ./dist (workspace root).
( cd frontend && trunk build )

# 3. Start the backend; it serves API / WS plus the dist directory.
cargo run -p foxide-app
# → http://127.0.0.1:3000
```

For an iterative loop, run `trunk watch` in one terminal and `cargo run -p foxide-app` in another.

`backend/src/lib.rs` reads the `DIST_DIR` env var (default `dist`) to locate the SPA bundle.

## Checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --exclude foxide-frontend --all-targets -- -D warnings
cargo clippy -p foxide-frontend --target wasm32-unknown-unknown --all-targets -- -D warnings
cargo test --workspace --exclude foxide-frontend
```

## CI

`.github/workflows/ci.yml`:

- **frontend-build** runs `./.github/actions/frontend-build` (composite: rust toolchain → trunk → fetch tailwindcss/daisyUI → `trunk build --release`) and uploads `dist/` as an artifact.
- **fmt** / **taplo** / **machete** / **unused-allow** are independent.
- **clippy** and **test** download the `dist` artifact, then run host clippy with `--exclude foxide-frontend`, plus a separate wasm32 clippy on the frontend; tests skip the frontend.
- **build-binary** downloads `dist` and runs `cargo build --release -p foxide-app`.
