//! secret アダプタで使う定数。

/// キーチェーン上の保存場所を表す識別子。
/// macOS では「サービス名 + アカウント名」のペアで1エントリが特定される。
pub(super) const SERVICE: &str = "com.yamaday.switchbotdesktop";
pub(super) const ACCOUNT: &str = "switchbot-api-credentials";
