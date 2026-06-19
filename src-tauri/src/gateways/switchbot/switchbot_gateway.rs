use async_trait::async_trait;
use thiserror::Error;

use crate::models::Credentials;

/// SwitchBot クラウド API への窓口（抽象 = trait）。
///
/// 具象実装は同じ `gateways/switchbot`（reqwest + HMAC 署名）に置く。
/// usecases はこの trait にだけ依存し、HTTP や署名の詳細は知らない。
#[async_trait]
pub trait SwitchBotGateway: Send + Sync {
    /// 資格情報の疎通検証（保存前に使う）。
    ///
    /// `GET /v1.1/devices` 相当を署名付きで呼び、成功なら `Ok(())`。
    /// - 認証失敗(401) → `Err(GatewayError::Unauthorized)`
    /// - 到達不可     → `Err(GatewayError::Network(..))`
    async fn validate_credentials(&self, creds: &Credentials) -> Result<(), GatewayError>;

    // NOTE: list_devices / get_status / send_command / scenes などは Epic B 以降で追加する。
}

/// Gateway 由来のエラー。
///
/// 要件 A3 の通り「認証失敗(401)」と「ネットワーク不通」を区別できることが重要。
#[derive(Debug, Error)]
pub enum GatewayError {
    /// トークン/シークレットが無効（HTTP 401 など）。
    #[error("認証に失敗しました（トークンまたはシークレットが無効です）")]
    Unauthorized,

    /// レート制限超過（HTTP 429）。
    #[error("APIのレート制限に達しました")]
    RateLimited,

    /// サーバに到達できない等のネットワークエラー（正否は不明）。
    #[error("ネットワークエラー: {0}")]
    Network(String),

    /// その他の予期しないエラー。
    #[error("予期しないエラー: {0}")]
    Unexpected(String),
}
