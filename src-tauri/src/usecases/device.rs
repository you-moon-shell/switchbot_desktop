use std::sync::Arc;

use thiserror::Error;

use crate::gateways::{GatewayError, SwitchBotGateway};
use crate::models::{Credential, Device, DeviceStatus};
use crate::repositories::{SecretRepository, SecretRepositoryError};

/// デバイス状態の取得ユースケース（要件 Epic B：読み取り専用）。
///
/// gateway に加えて `SecretRepository` も持つのが Epic A との違い。A の検証ではフォームから
/// 受け取った creds を使ったが、B では **保存済みの資格情報をキーチェーンからロードして** API を
/// 呼ぶ。具象には依存せず2つの抽象（trait）にだけ依存する。
pub struct DeviceUseCases {
    gateway: Arc<dyn SwitchBotGateway>,
    secrets: Arc<dyn SecretRepository>,
}

/// デバイスユースケースのエラー。
///
/// `CredentialError` と同じ方針：gateway 由来は**この層の語彙に平坦化**して持ち（内側の
/// `GatewayError` を抱えない）、保存層由来は1コードに畳むため `#[error(transparent)]` で透過保持する。
#[derive(Debug, Error)]
pub enum DeviceError {
    /// 認証失敗（API 401/403）、または資格情報が未保存。いずれも再認証導線へ（要件 B4）。
    #[error("認証に失敗しました（再認証が必要です）")]
    Unauthorized,

    /// API のレート制限超過（429）。
    #[error("APIのレート制限に達しました")]
    RateLimited,

    /// ネットワーク不通（正否は不明）。
    #[error("ネットワークエラー: {0}")]
    Network(String),

    /// その他の予期しない API エラー。
    #[error("予期しないエラー: {0}")]
    Unexpected(String),

    /// 保存層（キーチェーン）の読み取りに失敗。
    #[error(transparent)]
    Storage(#[from] SecretRepositoryError),
}

/// gateway の語彙 → usecase の語彙へ翻訳（境界での平坦化）。
impl From<GatewayError> for DeviceError {
    fn from(err: GatewayError) -> Self {
        match err {
            GatewayError::Unauthorized => Self::Unauthorized,
            GatewayError::RateLimited => Self::RateLimited,
            GatewayError::Network(msg) => Self::Network(msg),
            GatewayError::Unexpected(msg) => Self::Unexpected(msg),
        }
    }
}

impl DeviceUseCases {
    pub fn new(gateway: Arc<dyn SwitchBotGateway>, secrets: Arc<dyn SecretRepository>) -> Self {
        Self { gateway, secrets }
    }

    /// 保存済み資格情報をロードする。未保存なら `Unauthorized`（＝再認証導線へ）。
    ///
    /// 「未保存」と「API 401」はフロントの導線が同じ（要認証）なので、同じ variant に畳む。
    fn load_credential(&self) -> Result<Credential, DeviceError> {
        // load() は Result<Option<Credential>>。`?` で読み取り失敗（Err）を伝播し、残った
        // Option を `ok_or` で Result へ変換する: Some(creds)→Ok(creds) / None(未保存)→Err(Unauthorized)。
        // ＝「読めた中身があるか／無いか」を「成功か／要認証エラーか」に畳む1行。
        self.secrets.load()?.ok_or(DeviceError::Unauthorized)
    }

    /// 物理デバイス一覧を取得する（要件 B1）。
    pub async fn list_devices(&self) -> Result<Vec<Device>, DeviceError> {
        let creds = self.load_credential()?;
        Ok(self.gateway.list_devices(&creds).await?)
    }

    /// 指定デバイスの現在状態を取得する（要件 B2）。
    pub async fn get_status(&self, device_id: &str) -> Result<DeviceStatus, DeviceError> {
        let creds = self.load_credential()?;
        Ok(self.gateway.get_device_status(&creds, device_id).await?)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use serde_json::json;

    use super::*;
    use crate::models::DeviceKind;

    /// 偽の Gateway。一覧/状態の取得結果（成功 or エラー）を指定でき、呼ばれたかも記録する。
    struct FakeGateway {
        behavior: Behavior,
        called: Mutex<bool>,
    }

    enum Behavior {
        Success,
        Unauthorized,
        NetworkDown,
    }

    impl FakeGateway {
        fn new(behavior: Behavior) -> Self {
            Self {
                behavior,
                called: Mutex::new(false),
            }
        }
    }

    #[async_trait]
    impl SwitchBotGateway for FakeGateway {
        async fn validate_credential(&self, _creds: &Credential) -> Result<(), GatewayError> {
            unimplemented!("device ユースケースのテストでは使わない")
        }

        async fn list_devices(&self, _creds: &Credential) -> Result<Vec<Device>, GatewayError> {
            *self.called.lock().unwrap() = true;
            match self.behavior {
                Behavior::Success => Ok(vec![Device {
                    device_id: "AA".to_string(),
                    device_name: "リビングの温湿度計".to_string(),
                    kind: DeviceKind::Physical,
                    device_type: "Meter".to_string(),
                    hub_device_id: Some("HUB".to_string()),
                }]),
                Behavior::Unauthorized => Err(GatewayError::Unauthorized),
                Behavior::NetworkDown => Err(GatewayError::Network("offline".to_string())),
            }
        }

        async fn get_device_status(
            &self,
            _creds: &Credential,
            device_id: &str,
        ) -> Result<DeviceStatus, GatewayError> {
            *self.called.lock().unwrap() = true;
            match self.behavior {
                Behavior::Success => {
                    let fields = json!({ "temperature": 25.0, "humidity": 50, "battery": 100 })
                        .as_object()
                        .unwrap()
                        .clone();
                    Ok(DeviceStatus {
                        device_id: device_id.to_string(),
                        device_type: "Meter".to_string(),
                        fields,
                    })
                }
                Behavior::Unauthorized => Err(GatewayError::Unauthorized),
                Behavior::NetworkDown => Err(GatewayError::Network("offline".to_string())),
            }
        }
    }

    /// 偽の SecretRepository。キーチェーンの代わりにメモリ上の変数で保持する。
    #[derive(Default)]
    struct InMemorySecretRepository {
        saved: Mutex<Option<Credential>>,
    }

    impl InMemorySecretRepository {
        /// 「認証済み」状態（資格情報が保存済み）の repository を作る。
        fn authenticated() -> Self {
            Self {
                saved: Mutex::new(Some(Credential {
                    token: "tok".to_string(),
                    secret: "sec".to_string(),
                })),
            }
        }
    }

    impl SecretRepository for InMemorySecretRepository {
        fn save(&self, creds: &Credential) -> Result<(), SecretRepositoryError> {
            *self.saved.lock().unwrap() = Some(creds.clone());
            Ok(())
        }
        fn load(&self) -> Result<Option<Credential>, SecretRepositoryError> {
            Ok(self.saved.lock().unwrap().clone())
        }
        fn delete(&self) -> Result<(), SecretRepositoryError> {
            *self.saved.lock().unwrap() = None;
            Ok(())
        }
    }

    /// AC-B1: 認証済みなら一覧が取得できる。
    #[tokio::test]
    async fn list_devices_returns_devices_when_authenticated() {
        let usecase = DeviceUseCases::new(
            Arc::new(FakeGateway::new(Behavior::Success)),
            Arc::new(InMemorySecretRepository::authenticated()),
        );

        let devices = usecase.list_devices().await.unwrap();

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_type, "Meter");
    }

    /// B4: 資格情報が未保存なら API を呼ばずに Unauthorized（再認証導線へ）。
    #[tokio::test]
    async fn missing_credential_is_unauthorized_without_calling_api() {
        let gateway = Arc::new(FakeGateway::new(Behavior::Success));
        let usecase = DeviceUseCases::new(
            gateway.clone(),
            Arc::new(InMemorySecretRepository::default()),
        );

        let result = usecase.list_devices().await;

        assert!(matches!(result, Err(DeviceError::Unauthorized)));
        assert!(!*gateway.called.lock().unwrap());
    }

    /// B4: API 401 → Unauthorized に平坦化される。
    #[tokio::test]
    async fn gateway_unauthorized_maps_to_device_error_unauthorized() {
        let usecase = DeviceUseCases::new(
            Arc::new(FakeGateway::new(Behavior::Unauthorized)),
            Arc::new(InMemorySecretRepository::authenticated()),
        );

        let result = usecase.list_devices().await;

        assert!(matches!(result, Err(DeviceError::Unauthorized)));
    }

    /// B4: ネットワーク不通 → 401 と区別して Network になる。
    #[tokio::test]
    async fn gateway_network_error_is_distinguished() {
        let usecase = DeviceUseCases::new(
            Arc::new(FakeGateway::new(Behavior::NetworkDown)),
            Arc::new(InMemorySecretRepository::authenticated()),
        );

        let result = usecase.list_devices().await;

        assert!(matches!(result, Err(DeviceError::Network(_))));
    }

    /// AC-B2: 認証済みなら状態（種別別フィールド）が取得できる。
    #[tokio::test]
    async fn get_status_returns_fields_when_authenticated() {
        let usecase = DeviceUseCases::new(
            Arc::new(FakeGateway::new(Behavior::Success)),
            Arc::new(InMemorySecretRepository::authenticated()),
        );

        let status = usecase.get_status("AA").await.unwrap();

        assert_eq!(status.device_id, "AA");
        assert!(status.fields.contains_key("temperature"));
    }
}
