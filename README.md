# SwitchBot Desktop

SwitchBot 公式クラウド API を使って、デスクトップからデバイスを確認・操作する macOS 向けアプリ。
API トークン/シークレットは **OS キーチェーン**にのみ保管し、署名・通信はすべて **Rust コア**に閉じ込める設計。

> 🚧 開発中。現在 **Epic A（オンボーディング/認証）・Epic B（デバイス一覧・状態の読み取り表示）まで実装済み**。デバイス操作（ON/OFF など）は順次追加（→ [仕様](docs/spec/README.md)）。

## 技術スタック

| 領域         | 採用                                                                             |
| ------------ | -------------------------------------------------------------------------------- |
| アプリ基盤   | [Tauri v2](https://tauri.app/)（Rust コア + WebView）                            |
| フロント     | React 19 + TypeScript + Vite / TanStack Query / React Router v7（memory router） |
| スタイル     | Tailwind CSS v4 ＋ 自作グラスモーフィズム UI（ダーク固定）                       |
| バックエンド | Rust（reqwest + HMAC 署名、keyring によるキーチェーン保管）                      |

## アーキテクチャ

依存性逆転（trait 境界）＋ 責務別構成。詳細は仕様を参照：

- [📁 仕様インデックス](docs/spec/README.md)
- [バックエンド（Rust コア）](docs/spec/architecture/backend.md)
- [フロントエンド（React / WebView）](docs/spec/architecture/frontend.md)

## 開発

### セットアップ & 起動

```sh
pnpm install
pnpm tauri dev      # ネイティブウィンドウで起動（ホットリロード）
```

### よく使うコマンド

| 目的                           | コマンド                                    |
| ------------------------------ | ------------------------------------------- |
| フロント開発サーバ（ブラウザ） | `pnpm dev`                                  |
| 本番ビルド（アプリ）           | `pnpm tauri build`                          |
| Rust テスト                    | `cd src-tauri && cargo test`                |
| Rust 整形 / Lint               | `cd src-tauri && cargo fmt && cargo clippy` |
| 型チェック（フロント）         | `pnpm exec tsc --noEmit`                    |
