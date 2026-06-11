use serde::{Deserialize, Serialize};

/// SwitchBot API の資格情報（公開トークンと署名鍵）。
///
/// Debug は自動導出せず、秘密を伏せた独自実装にしている（`{:?}` でログに出ても漏れない）。
#[derive(Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// 公開トークン（Authorization ヘッダに使用）。
    pub token: String,
    /// 署名鍵（HMAC-SHA256 の鍵。送信はしない）。
    pub secret: String,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("token", &"***")
            .field("secret", &"***")
            .finish()
    }
}
