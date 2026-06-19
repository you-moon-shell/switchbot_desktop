---
paths:
  - "src-tauri/**/*.rs"
  - "src/**/*.{ts,tsx}"
---

# セキュリティ方針（厳守）

根拠は [backend spec](../../docs/spec/architecture/backend.md) §7 ／ [Epic A requirements](../../docs/spec/epics/a-onboarding-auth/requirements.md) NFR。

- **機密（トークン/シークレット）は OS キーチェーンにのみ保存**する。WebView・ログ・DB・設定ファイルへは一切書き出さない。
- **HMAC 署名・HTTP 通信・キーチェーン操作はすべて Rust コア内**で行う。フロントは SwitchBot API を直接呼ばない。
- **フロント↔コアの通信は `invoke` / `listen` の 2 本のみ**。機密そのものはフロントへ渡さない（`has_credentials` のように bool 等に留める）。
- ログ出力時は Token/Secret を**マスキング**する（`Credentials` の Debug は `***` で伏せる実装を維持）。
