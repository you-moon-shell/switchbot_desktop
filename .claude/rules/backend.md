---
paths:
  - "src-tauri/**/*.rs"
---

# バックエンド（Rust コア）コーディングルール

根拠・詳細は [backend spec](../../docs/spec/architecture/backend.md)。

## 依存の向き（依存性逆転）

- `commands → usecases → trait`。**usecases は具象（keyring / reqwest 実装）に依存しない**——外界は `gateways` / `repositories` の trait 越しに使う。
- 具象型名（`KeyringSecretRepository` / `SwitchBotApiGateway`）が登場するのは `lib.rs` の DI 配線だけ。

## エラー設計

- 各層が自分のエラー型を持ち、**境界で翻訳**する。内側の型を外へ漏らさない（例: usecase は `GatewayError` を `CredentialError` の自層バリアントに**平坦化**して持つ）。
- 「呼び出し側が**分岐に使う**」情報は enum バリアントへ昇格、「読むだけ」の詳細は `String` に入れる。
- 実装詳細のエラー（reqwest / keyring）は具象で翻訳し、trait の外へ出さない。

## 構成・命名

- 外界は責務別：`gateways/`（外部API）・`repositories/`（永続化）。各責務フォルダに**契約 trait と具象を別ファイルで同居**（1 ファイル = 1 型）。
- trait の非同期メソッドは `#[async_trait]`。DI は `lib.rs` の Composition Root で手配線する。
- 定数は各モジュールの `constants.rs`（公開範囲は `pub(super)`）。
