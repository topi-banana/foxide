# CLAUDE.md

このファイルは Claude Code (claude.ai/code) がこのリポジトリで作業する際のガイドです。

## プロジェクト概要

[Yew](https://yew.rs) (CSR、struct ベースの Component) と Axum で構築されたファイルブラウザ。フロントエンドのバンドルは [Trunk](https://trunkrs.dev) で生成し、Axum が workspace ルート直下の `dist/` を配信する。

Rust 2024 edition のワークスペースで、4つのクレートから構成される:

| ディレクトリ | クレート               | 役割                                                       |
| ------------ | ---------------------- | ---------------------------------------------------------- |
| `app/`       | `filebrowser-app`      | サーバー起動 (Axum、API/WS + SPA 配信)                     |
| `frontend/`  | `filebrowser-frontend` | Yew 製 UI、ルーター、`trunk` のビルドエントリ              |
| `backend/`   | `filebrowser-backend`  | API/WS ハンドラ、永続化                                    |
| `types/`     | `filebrowser-types`    | 共有型定義                                                 |

## 重要な設計

- `App` (`frontend/src/lib.rs`) が一番上の Component。`Component::rendered(first_render=true)` で WebSocket を 1 回だけ open し、`Closure<dyn FnMut(MessageEvent)>` を保持する。
- `App` は受信メッセージを `update()` で `AppData` に集約し、`AppCtx` を `ContextProvider` 経由で子 Component に配る。子 Component は `ctx.link().context::<AppCtx>(...)` で購読する。
- すべての Component は `impl yew::Component for ...` を直接書く方式 (関数 Component / `#[function_component]` は使わない)。
- `frontend/Trunk.toml` で `dist = "../dist"` を指定し、`pre_build` フックで `./tailwindcss -i input.css -o output.css` を実行する。`tailwindcss` バイナリと `daisyui.mjs` / `daisyui-theme.mjs` は gitignore 済みで、CI とローカルでビルド時にダウンロードする。

## 前提条件

- [Rust](https://rustup.rs/) (edition 2024)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`
- `frontend/tailwindcss` (Tailwind v4 standalone CLI) と `frontend/daisyui.mjs` / `frontend/daisyui-theme.mjs` (CI と同じく `tailwindlabs/tailwindcss` と `saadeghi/daisyui` の latest release から取得)

## 開発コマンド

```bash
# 1. tailwindcss と daisyUI を取得 (初回のみ)
cd frontend
curl -sLo tailwindcss https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64
curl -sLO https://github.com/saadeghi/daisyui/releases/latest/download/daisyui.mjs
curl -sLO https://github.com/saadeghi/daisyui/releases/latest/download/daisyui-theme.mjs
chmod +x tailwindcss
cd ..

# 2. SPA バンドル生成 (workspace ルート直下の dist/ に出力)
( cd frontend && trunk build )

# 3. サーバー起動 (http://127.0.0.1:3000)
cargo run -p filebrowser-app

# 開発中: ファイル変更で再ビルド
( cd frontend && trunk watch ) &
cargo run -p filebrowser-app
```

### Lint / フォーマット / テスト

```bash
cargo fmt --all -- --check
taplo fmt --check
cargo-unused-allow --all-targets -- --workspace
cargo clippy --workspace --exclude filebrowser-frontend --all-targets -- -D warnings
cargo clippy -p filebrowser-frontend --target wasm32-unknown-unknown --all-targets -- -D warnings
cargo test --workspace --exclude filebrowser-frontend
```

### 全チェックを一括実行

```bash
cargo fmt --all -- --check \
  && taplo fmt --check \
  && cargo-unused-allow --all-targets -- --workspace \
  && cargo clippy --workspace --exclude filebrowser-frontend --all-targets -- -D warnings \
  && cargo clippy -p filebrowser-frontend --target wasm32-unknown-unknown --all-targets -- -D warnings \
  && cargo test --workspace --exclude filebrowser-frontend
```

## CI

GitHub Actions (`.github/workflows/ci.yml`) は以下のジョブを実行する:

- **frontend-build** — `./.github/actions/frontend-build` (composite) で trunk + tailwindcss + daisyUI を実行し、`dist/` を artifact として upload。
- **fmt** — `cargo fmt --all -- --check`
- **taplo** — `taplo fmt --check`
- **clippy** — frontend を除外した host clippy と、wasm32 ターゲットの frontend clippy。`dist/` artifact を download してから走る (一部クレートが `dist/index.html` を実行時に参照するため、整合性確保のため artifact を共有)。
- **unused-allow** — 未使用 `#[allow(...)]` の検出。
- **test** — `cargo test --workspace --exclude filebrowser-frontend`。`dist/` artifact をダウンロード。
- **machete** — `bnjbvr/cargo-machete` で未使用依存を検出。
- **build-binary** — `cargo build --release -p filebrowser-app`。

## 注意事項

- WebSocket の open は `App::rendered(_, first_render)` の `first_render == true` の枝でのみ行う (二重接続を作らない)。
- ページ Component が WS リクエストを送るときは、`AppCtx::send: Callback<ClientMsg>` を使う。WS 自体は `App` だけが保持する。
- ページ Component は `ContextChanged` 経由で `AppCtx` の更新を受け、必要なら `AppData::*::seq` の変化で「新着レスポンス」を検出する (`volume::VolumePage::last_browse_seq` を参照)。
- バックエンドが配信する SPA バンドルのパスは `DIST_DIR` 環境変数で上書き可能 (デフォルト `dist`)。`backend/src/lib.rs` の `DEFAULT_DIST_DIR` を参照。
- `types` クレートは `backend` と `frontend` の共有依存として型定義を一元管理する。
