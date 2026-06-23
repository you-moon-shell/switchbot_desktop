use serde::{Deserialize, Serialize};
use thiserror::Error;

/// SwitchBot API の資格情報（公開トークンと署名鍵）。
///
/// Debug は自動導出せず、秘密を伏せた独自実装にしている（`{:?}` でログに出ても漏れない）。
/// 生入力からの構築は `from_input` 経由を想定し、「保存・署名に使う Credential は
/// 常にトリム済み・非空」という不変条件を型で守る。
#[derive(Clone, Serialize, Deserialize)]
pub struct Credential {
    /// 公開トークン（Authorization ヘッダに使用）。
    pub token: String,
    /// 署名鍵（HMAC-SHA256 の鍵。送信はしない）。
    pub secret: String,
}

/// `Credential::from_input` の失敗。トリム後に token / secret のどちらかが空だった、という
/// モデル層の最小エラー。usecase など上位層が自層の語彙へ翻訳して扱う（境界で翻訳）。
#[derive(Debug, Error)]
#[error("token と secret は空にできません")]
pub struct EmptyCredential;

impl Credential {
    /// 生入力を正規化（前後の空白をトリム）して構築する。
    /// トリム後に token / secret のどちらかが空なら `EmptyCredential` を返す。
    /// これにより不正な状態（未トリム・空）の Credential を作れないようにする。
    pub fn from_input(token: &str, secret: &str) -> Result<Self, EmptyCredential> {
        let token = token.trim().to_string();
        let secret = secret.trim().to_string();
        if token.is_empty() || secret.is_empty() {
            return Err(EmptyCredential);
        }
        Ok(Self { token, secret })
    }
}

impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credential")
            .field("token", &"***")
            .field("secret", &"***")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_input_trims_surrounding_whitespace() {
        let creds = Credential::from_input("  tok  ", "\tsec\n").unwrap();
        assert_eq!(creds.token, "tok");
        assert_eq!(creds.secret, "sec");
    }

    #[test]
    fn from_input_rejects_blank_after_trim() {
        assert!(Credential::from_input("   ", "sec").is_err());
        assert!(Credential::from_input("tok", " \t ").is_err());
    }
}
