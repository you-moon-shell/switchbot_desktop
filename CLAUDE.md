# CLAUDE.md

このリポジトリで作業するときの前提とガイド。

## ユーザーの前提（**重要**）

- **Rust 初心者**。構文・イディオム・標準ライブラリの概念は、実装の流れの中で**平易な言葉で説明**する。例: `Arc<dyn Trait>`、`?` 演算子、所有権/借用、`Result`/`Option`、`#[from]`、ライフタイムなど。TypeScript との対比を添えると伝わりやすい（trait ≒ interface、`Arc<dyn T>` ≒ interface 型で受ける感覚、など）。
- **ソフトウェアアーキテクチャを学習中**。設計の変更・判断は「何をするか」だけでなく、**「なぜそうするのか／他の選択肢との違い／崩すと何が困るか」**まで説明する。レイヤリング、依存の向き（内向き）、依存性逆転、テスト容易性といった観点を明示する。
- 黙って実装だけ進めない。変更の意図と「ここから学べること」を一言添える。設計の岐路では選択肢と推奨理由を示す。

## アーキテクチャ

- 仕様の入口（索引）: `docs/spec/README.md`
- バックエンド（Rust コア）の設計: `docs/spec/architecture/backend.md`
- フロントエンド（React/TS）の設計: `docs/spec/architecture/frontend.md`
- スタイルは **依存性逆転（trait 境界）＋ 責務別構成**。
  - `usecases/` … アプリのロジック。**trait にだけ依存**し、具象実装は知らない。
  - `gateways/` … 外部API（SwitchBot クラウド）への窓口。契約 trait と具象を同居。
  - `repositories/` … 永続化（OSキーチェーン／将来 SQLite）。契約 trait と具象を同居。
  - `models/` … 純粋な型。`commands/` … invoke 受け口。`lib.rs` … DI 配線（Composition Root）。
- 依存の向き: `commands → usecases → trait`。具象（keyring/reqwest 実装）へは向かない。

## テスト方針

**ビジネスロジックと特殊／非自明な計算ロジックには必ずテストを書く。**

- **書く（必須）**
  - **ビジネスロジック** = usecases の判断・分岐・バリデーション・オーケストレーション（例: 「検証成功時のみ保存」「空入力は API を呼ばず弾く」「前後空白のトリム」）。外界は trait の Fake / InMemory 実装を注入し、実ネットワーク・実キーチェーン無しで検証する（例: `usecases/credential.rs` の単体テスト）。
  - **特殊／非自明な計算** = 署名・暗号・日時/単位変換・状態遷移・パースなど、間違えやすく仕様が定まっているもの。**純粋関数に切り出して**入力→期待値で固定する（例: `gateways/switchbot/signature.rs` の `compute_sign` を公式サンプルと一致検証）。
- **必須ではない（ロジックが無ければ書かない）**
  - 薄い委譲（`commands` の usecase 呼び出し）、DI 配線（`lib.rs`）、外界 IO ラッパそのもの（keyring/reqwest を呼ぶだけの具象）。これらは OS/ネットワークのモックが必要な割に守る対象が薄い。IO は usecase 側の Fake テストで間接的にカバーする。
- **テスト容易性は設計で確保する**：分岐に関わる純粋ロジックは IO から分離し、外界は trait 化して差し替え可能にする（テストのために設計を歪めるのではなく、良い設計が自然にテスト可能になる形を狙う）。

## 開発コマンド（Rust コア = `src-tauri/`）

- ビルド: `cargo build`
- テスト: `cargo test`
- 整形: `cargo fmt`
- Lint: `cargo clippy`

> いずれも `src-tauri/` ディレクトリで実行する。
