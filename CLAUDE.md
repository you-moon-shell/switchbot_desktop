# CLAUDE.md

このリポジトリで作業するときの前提とガイド。

## ユーザーの前提（**重要**）

- **Rust 初心者**。構文・イディオム・標準ライブラリの概念は、実装の流れの中で**平易な言葉で説明**する。例: `Arc<dyn Trait>`、`?` 演算子、所有権/借用、`Result`/`Option`、`#[from]`、ライフタイムなど。TypeScript との対比を添えると伝わりやすい（trait ≒ interface、`Arc<dyn T>` ≒ interface 型で受ける感覚、など）。
- **ソフトウェアアーキテクチャを学習中**。設計の変更・判断は「何をするか」だけでなく、**「なぜそうするのか／他の選択肢との違い／崩すと何が困るか」**まで説明する。レイヤリング、依存の向き（内向き）、依存性逆転、テスト容易性といった観点を明示する。
- 黙って実装だけ進めない。変更の意図と「ここから学べること」を一言添える。設計の岐路では選択肢と推奨理由を示す。

## アーキテクチャ

- バックエンド（Rust コア）の設計: `docs/spec/backend-architecture.md`
- フロントエンド（React/TS）の設計: `docs/spec/frontend-architecture.md`
- スタイルは **依存性逆転（trait 境界）＋ 責務別構成**。
  - `usecases/` … アプリのロジック。**trait にだけ依存**し、具象実装は知らない。
  - `gateways/` … 外部API（SwitchBot クラウド）への窓口。契約 trait と具象を同居。
  - `repositories/` … 永続化（OSキーチェーン／将来 SQLite）。契約 trait と具象を同居。
  - `models/` … 純粋な型。`commands/` … invoke 受け口。`lib.rs` … DI 配線（Composition Root）。
- 依存の向き: `commands → usecases → trait`。具象（keyring/reqwest 実装）へは向かない。

## 開発コマンド（Rust コア = `src-tauri/`）

- ビルド: `cargo build`
- テスト: `cargo test`
- 整形: `cargo fmt`
- Lint: `cargo clippy`

> いずれも `src-tauri/` ディレクトリで実行する。
