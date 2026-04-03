# CLAUDE.md

このファイルは Claude Code (claude.ai/code) がこのリポジトリで作業する際のガイドです。

## プロジェクト概要

Leptos (SSR + Hydration) と Axum で構築されたファイルブラウザ。

Rust 2024 edition のワークスペースで、4つのクレートから構成される:

| ディレクトリ | クレート               | 役割                                         |
| ------------ | ---------------------- | -------------------------------------------- |
| `app/`       | `filebrowser-app`      | サーバー起動 (Axum + Leptos SSR)             |
| `frontend/`  | `filebrowser-frontend` | UIコンポーネント、Router、hydration エントリ |
| `backend/`   | `filebrowser-backend`  | APIエンドポイント、サーバーファンクション    |
| `types/`     | `filebrowser-types`    | 共有型定義                                   |

## 前提条件

- [Rust](https://rustup.rs/) (edition 2024)
- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) (ホットリロード付き開発サーバー用)

```bash
cargo install cargo-leptos
```

## 開発コマンド

```bash
# 開発サーバー起動 (ホットリロード付き、http://127.0.0.1:3000)
cargo leptos watch

# フォーマットチェック
cargo fmt --all -- --check
taplo fmt --check

# 未使用 allow 検出
cargo-unused-allow --all-targets -- --workspace

# Lint (SSR側)
cargo clippy -p filebrowser-app --features ssr

# Lint (hydrate/WASM側)
cargo clippy -p filebrowser-frontend --features hydrate

# テスト (サーバー起動してAPIエンドポイントをテスト)
cargo test -p filebrowser-app --features ssr

# backend/types テスト
cargo test -p filebrowser-backend
cargo test -p filebrowser-types
```

### 全チェックを一括実行

```bash
cargo fmt --all -- --check \
  && taplo fmt --check \
  && cargo-unused-allow --all-targets -- --workspace \
  && cargo clippy -p filebrowser-app --features ssr \
  && cargo clippy -p filebrowser-frontend --features hydrate \
  && cargo test -p filebrowser-app --features ssr \
  && cargo test -p filebrowser-backend \
  && cargo test -p filebrowser-types
```

## CI

GitHub Actions が全ブランチへの push / PR ごとに以下を実行:

- **Format** — `cargo fmt --check`
- **Clippy** — SSR と hydrate 両方の lint
- **Unused Allow** — 未使用 `#[allow(...)]` の検出 (`cargo-unused-allow`)
- **Test** — サーバーを起動してAPIエンドポイントを検証する統合テスト
- **Machete** — 未使用依存の検出 (`cargo-machete`)
- **Build** — フル SSR ビルド

## 注意事項

- Clippy はワークスペース全体ではなく、SSR (`--features ssr`) と hydrate (`--features hydrate`) で分けて実行する
- テスト時も `--features ssr` フラグが必要なクレートがある
- `types` クレートは `backend` と `frontend` の共有依存として型定義を一元管理する
