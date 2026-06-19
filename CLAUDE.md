# CLAUDE.md

このリポジトリで作業する AI 向けの指針。

- プロジェクト概要・技術スタック・コマンド … [README](README.md)
- 設計の詳細 … [docs/spec](docs/spec/README.md)
- コード固有のルール … `.claude/rules/`（`backend.md`=Rust / `frontend.md`=フロント / `testing.md`=テスト。編集対象に応じて自動ロード）

ここ（CLAUDE.md）は前提と横断的な進め方に絞り、上記と重複させない。

## ユーザーの前提（**重要**）

- **Rust 初心者**。構文・イディオム・標準ライブラリの概念は、実装の流れの中で**平易な言葉で説明**する（例: `Arc<dyn Trait>`、`?` 演算子、所有権/借用、`Result`/`Option`、`#[from]`、ライフタイム）。TypeScript との対比を添えると伝わりやすい（trait ≒ interface、`Arc<dyn T>` ≒ interface 型で受ける感覚、など）。
- **ソフトウェアアーキテクチャを学習中**。設計の判断は「何をするか」だけでなく **「なぜそうするのか／他の選択肢との違い／崩すと何が困るか」** まで説明する。
- 黙って実装だけ進めない。変更の意図と「ここから学べること」を一言添え、設計の岐路では選択肢と推奨理由を示す。

## 進め方の規約

- **仕様フロー**：エピックは `docs/spec/epics/<id>/requirements.md` のみ。design.md / tasks.md は作らず、requirements をベースに**コーディングしながら調整**する。
- **変更後の確認**：Rust は `src-tauri/` で `cargo fmt && cargo clippy && cargo test`、フロントは `pnpm exec tsc --noEmit` を通す。
