use std::time::Duration;

use async_trait::async_trait;

use super::constants::BASE_URL;
use super::signature::build_signed_headers;
use super::{GatewayError, SwitchBotGateway};
use crate::models::Credentials;

/// SwitchBot API の共通レスポンス（必要なフィールドだけを写し取る）。
/// 仕様: 成功時は statusCode = 100。それ以外はエラー。
#[derive(serde::Deserialize)]
struct ApiResponse {
    #[serde(rename = "statusCode")]
    status_code: i64,
}

/// `SwitchBotGateway` の本番実装（reqwest + HMAC-SHA256 署名）。
pub struct SwitchBotApiGateway {
    http: reqwest::Client,
}

impl SwitchBotApiGateway {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("HTTPクライアントの初期化に失敗");
        Self { http }
    }
}

#[async_trait]
impl SwitchBotGateway for SwitchBotApiGateway {
    async fn validate_credentials(&self, creds: &Credentials) -> Result<(), GatewayError> {
        let headers = build_signed_headers(creds);
        let resp = self
            .http
            .get(format!("{BASE_URL}/devices"))
            .header("Authorization", &headers.authorization)
            .header("sign", &headers.sign)
            .header("t", &headers.t)
            .header("nonce", &headers.nonce)
            .header("Content-Type", "application/json; charset=utf8")
            .send()
            .await
            .map_err(|e| GatewayError::Network(e.to_string()))?;

        match resp.status().as_u16() {
            200 => {
                // HTTP 200 でも body の statusCode が 100 以外なら成功ではない
                let body: ApiResponse = resp
                    .json()
                    .await
                    .map_err(|e| GatewayError::Unexpected(e.to_string()))?;
                match body.status_code {
                    100 => Ok(()),
                    code => Err(GatewayError::Unexpected(format!(
                        "APIエラー (statusCode: {code})"
                    ))),
                }
            }
            401 | 403 => Err(GatewayError::Unauthorized),
            429 => Err(GatewayError::RateLimited),
            status => Err(GatewayError::Unexpected(format!("HTTP {status}"))),
        }
    }
}
