//! SwitchBot API v1.1 の HMAC-SHA256 署名。
//!
//! 仕様: `sign = upper(base64(HMAC-SHA256(key=secret, msg=token + t + nonce)))`
//! - `t` は13桁のミリ秒タイムスタンプ
//! - `nonce` はリクエスト毎にユニークな UUID v4
//! - リクエストボディは署名に含めない

use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

use crate::models::Credential;

type HmacSha256 = Hmac<Sha256>;

pub(super) struct SignedHeaders {
    pub(super) authorization: String,
    pub(super) sign: String,
    pub(super) t: String,
    pub(super) nonce: String,
}

/// 現在時刻と新規 nonce で署名ヘッダを組み立てる。
pub(super) fn build_signed_headers(creds: &Credential) -> SignedHeaders {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("システム時計が1970年より前になっている")
        .as_millis()
        .to_string();
    let nonce = Uuid::new_v4().to_string();
    let sign = compute_sign(&creds.token, &creds.secret, &t, &nonce);

    SignedHeaders {
        authorization: creds.token.clone(),
        sign,
        t,
        nonce,
    }
}

/// 署名の純粋な計算部分（時刻や乱数を含まないので単体テスト可能）。
fn compute_sign(token: &str, secret: &str, t: &str, nonce: &str) -> String {
    let string_to_sign = format!("{token}{t}{nonce}");
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMACは任意長の鍵を受け付けるため失敗しない");
    mac.update(string_to_sign.as_bytes());
    STANDARD
        .encode(mac.finalize().into_bytes())
        .to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 公式README記載のPythonサンプルと同一手順で算出した期待値と一致すること。
    #[test]
    fn compute_sign_matches_official_python_sample() {
        let sign = compute_sign("test-token", "test-secret", "1700000000000", "fixed-nonce");
        assert_eq!(sign, "PT+65CGDGEAPQKE0DGVH+8CEY0NNQQHAYUDPZZBXVJQ=");
    }
}
