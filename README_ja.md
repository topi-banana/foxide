# Foxide

**Foxide** は Rust で書かれたセルフホスト型のファイルブラウザです。名前は **file** + **oxide** の合成語で、Rust エコシステム (酸化物 = oxide のもじり) を意識しつつ「狐 (fox) 風」の親しみやすさを兼ねています。

フロントエンドは [Yew](https://yew.rs) (struct ベースの Component、[Trunk](https://trunkrs.dev) による CSR)、バックエンドは Axum で実装されています。単一の Axum サーバーが WebSocket / API エンドポイントと SPA バンドルの配信を兼ねます。

[English README is here.](./README.md)

## 前提条件

- [Rust](https://rustup.rs/) (edition 2024)
- `wasm32-unknown-unknown` ターゲット: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`
- `frontend/tailwindcss` (Tailwind v4 standalone CLI) と daisyUI プラグイン (`frontend/daisyui.mjs`, `frontend/daisyui-theme.mjs`)。`frontend/Trunk.toml` の `pre_build` フックが `./tailwindcss -i input.css -o output.css` を自動実行します。

## プロジェクト構成

| ディレクトリ | クレート          | 役割                                                       |
| ------------ | ----------------- | ---------------------------------------------------------- |
| `app/`       | `foxide-app`      | サーバー起動 (Axum、API/WS + SPA 配信)                     |
| `frontend/`  | `foxide-frontend` | Yew 製 UI、ルーター、`trunk` のビルドエントリ              |
| `backend/`   | `foxide-backend`  | API/WS ハンドラ、永続化                                    |
| `types/`     | `foxide-types`    | 共有型定義                                                 |

ツリーの最上位にある Yew の `App` Component (`frontend/src/lib.rs`) が `Component::rendered(first_render=true)` で WebSocket 接続を 1 度だけ open し、`ContextProvider<AppCtx>` 経由で子の Page Component にメッセージを伝搬します。

## 開発手順

```bash
# 1. tailwindcss と daisyUI を取得 (初回のみ)
cd frontend
curl -sLo tailwindcss https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64
curl -sLO https://github.com/saadeghi/daisyui/releases/latest/download/daisyui.mjs
curl -sLO https://github.com/saadeghi/daisyui/releases/latest/download/daisyui-theme.mjs
chmod +x tailwindcss
cd ..

# 2. SPA バンドルをビルド (workspace ルート直下の dist/ に出力)
( cd frontend && trunk build )

# 3. バックエンドを起動 (API / WS と dist/ を配信)
cargo run -p foxide-app
# → http://127.0.0.1:3000
```

開発中は片方のターミナルで `trunk watch`、もう片方で `cargo run -p foxide-app` を走らせると変更が即時反映されます。

SPA バンドルのパスは `DIST_DIR` 環境変数で上書き可能です (デフォルト `dist`)。`backend/src/lib.rs` の `DEFAULT_DIST_DIR` を参照してください。

## チェック

```bash
cargo fmt --all -- --check
cargo clippy --workspace --exclude foxide-frontend --all-targets -- -D warnings
cargo clippy -p foxide-frontend --target wasm32-unknown-unknown --all-targets -- -D warnings
cargo test --workspace --exclude foxide-frontend
```

## CI

`.github/workflows/ci.yml` のジョブ:

- **frontend-build** — `./.github/actions/frontend-build` (composite: Rust toolchain → trunk → tailwindcss/daisyUI 取得 → `trunk build --release`) を実行し、`dist/` を artifact として upload。
- **fmt** / **taplo** / **machete** / **unused-allow** は独立。
- **clippy** と **test** は `dist` artifact をダウンロードしてから、host clippy (`--exclude foxide-frontend`) と wasm32 ターゲットの frontend clippy を実行。テストでは frontend を除外。
- **build-binary** は `dist` を取得した上で `cargo build --release -p foxide-app` を実行。
