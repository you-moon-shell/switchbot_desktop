use async_trait::async_trait;
use thiserror::Error;

use crate::models::{Credentials, Device, DeviceStatus};

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

    /// 物理デバイス一覧を取得する（要件 B1）。`GET /v1.1/devices`。
    ///
    /// 赤外線リモコン（`infraredRemoteList`）は対象外なので含めない。
    async fn list_devices(&self, creds: &Credentials) -> Result<Vec<Device>, GatewayError>;

    /// 1台の現在状態を取得する（要件 B2）。`GET /v1.1/devices/{deviceId}/status`。
    async fn get_device_status(
        &self,
        creds: &Credentials,
        device_id: &str,
    ) -> Result<DeviceStatus, GatewayError>;

    // NOTE: send_command / scenes などは Epic C 以降で追加する。
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
