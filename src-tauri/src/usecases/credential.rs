use std::sync::Arc;

use thiserror::Error;

use crate::models::Credentials;
use crate::ports::{GatewayError, SecretStore, SecretStoreError, SwitchBotGateway};

/// 認証まわりのユースケース（要件 Epic A）。
///
/// 具象には依存せず、2つの port（trait）にだけ依存する。
/// 本番は SwitchBotApiGateway + KeyringSecretStore、テストは Fake を注入する。
pub struct CredentialUseCases {
    gateway: Arc<dyn SwitchBotGateway>,
    secrets: Arc<dyn SecretStore>,
}

/// 認証ユースケースのエラー。
#[derive(Debug, Error)]
pub enum CredentialError {
    /// 入力が空（トリム後）。API を呼ぶまでもなく弾く。
    #[error("トークンとシークレットを入力してください")]
    EmptyInput,

    /// API 検証に失敗（401・ネットワーク等。中のエラーをそのまま透過表示）。
    #[error(transparent)]
    Gateway(#[from] GatewayError),

    /// キーチェーン操作に失敗。
    #[error(transparent)]
    Secret(#[from] SecretStoreError),
}

impl CredentialUseCases {
    pub fn new(gateway: Arc<dyn SwitchBotGateway>, secrets: Arc<dyn SecretStore>) -> Self {
        Self { gateway, secrets }
    }

    /// 入力された資格情報を検証し、成功した場合のみ保存する（要件 A3/A4）。
    ///
    /// - 前後の空白はトリムする（要件エッジケース）
    /// - 空入力は API を呼ばずに弾く
    /// - 検証失敗（401/ネットワーク）時は一切保存しない
    pub async fn validate_and_store(&self, token: &str, secret: &str) -> Result<(), CredentialError> {
        let creds = Credentials {
            token: token.trim().to_string(),
            secret: secret.trim().to_string(),
        };
        if creds.token.is_empty() || creds.secret.is_empty() {
            return Err(CredentialError::EmptyInput);
        }

        self.gateway.validate_credentials(&creds).await?;
        self.secrets.save(&creds)?;
        Ok(())
    }

    /// 資格情報が保存済みかどうか（要件 A1/A5 の起動時判定）。
    ///
    /// フロントには bool しか返さない（資格情報そのものは渡さない）。
    pub fn has_credentials(&self) -> Result<bool, CredentialError> {
        Ok(self.secrets.load()?.is_some())
    }

    /// ログアウト＝資格情報を削除する（要件 A6）。
    pub fn logout(&self) -> Result<(), CredentialError> {
        self.secrets.delete()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;

    use super::*;

    /// 偽の Gateway。挙動（成功/401/ネット断）を指定でき、呼ばれたかも記録する。
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
        async fn validate_credentials(&self, _creds: &Credentials) -> Result<(), GatewayError> {
            *self.called.lock().unwrap() = true;
            match self.behavior {
                Behavior::Success => Ok(()),
                Behavior::Unauthorized => Err(GatewayError::Unauthorized),
                Behavior::NetworkDown => Err(GatewayError::Network("offline".to_string())),
            }
        }
    }

    /// 偽の SecretStore。キーチェーンの代わりにメモリ上の変数に保存する。
    #[derive(Default)]
    struct InMemorySecretStore {
        saved: Mutex<Option<Credentials>>,
    }

    impl SecretStore for InMemorySecretStore {
        fn save(&self, creds: &Credentials) -> Result<(), SecretStoreError> {
            *self.saved.lock().unwrap() = Some(creds.clone());
            Ok(())
        }
        fn load(&self) -> Result<Option<Credentials>, SecretStoreError> {
            Ok(self.saved.lock().unwrap().clone())
        }
        fn delete(&self) -> Result<(), SecretStoreError> {
            *self.saved.lock().unwrap() = None;
            Ok(())
        }
    }

    /// AC-1相当: 正しい資格情報 → 保存される。前後の空白はトリムされる。
    #[tokio::test]
    async fn valid_credentials_are_trimmed_and_saved() {
        let store = Arc::new(InMemorySecretStore::default());
        let usecase = CredentialUseCases::new(
            Arc::new(FakeGateway::new(Behavior::Success)),
            store.clone(),
        );

        usecase
            .validate_and_store("  tok  ", "\tsec\n")
            .await
            .unwrap();

        let saved = store.saved.lock().unwrap().clone().unwrap();
        assert_eq!(saved.token, "tok");
        assert_eq!(saved.secret, "sec");
    }

    /// AC-2: 401 → エラーになり、保存されない。
    #[tokio::test]
    async fn unauthorized_credentials_are_not_saved() {
        let store = Arc::new(InMemorySecretStore::default());
        let usecase = CredentialUseCases::new(
            Arc::new(FakeGateway::new(Behavior::Unauthorized)),
            store.clone(),
        );

        let result = usecase.validate_and_store("tok", "sec").await;

        assert!(matches!(
            result,
            Err(CredentialError::Gateway(GatewayError::Unauthorized))
        ));
        assert!(store.saved.lock().unwrap().is_none());
    }

    /// AC-5相当: ネットワーク断 → 401 とは別のエラーになり、保存されない。
    #[tokio::test]
    async fn network_error_is_distinguished_and_not_saved() {
        let store = Arc::new(InMemorySecretStore::default());
        let usecase = CredentialUseCases::new(
            Arc::new(FakeGateway::new(Behavior::NetworkDown)),
            store.clone(),
        );

        let result = usecase.validate_and_store("tok", "sec").await;

        assert!(matches!(
            result,
            Err(CredentialError::Gateway(GatewayError::Network(_)))
        ));
        assert!(store.saved.lock().unwrap().is_none());
    }

    /// エッジケース: 空入力は API を呼ばずに弾く。
    #[tokio::test]
    async fn empty_input_is_rejected_without_calling_api() {
        let gateway = Arc::new(FakeGateway::new(Behavior::Success));
        let usecase = CredentialUseCases::new(
            gateway.clone(),
            Arc::new(InMemorySecretStore::default()),
        );

        let result = usecase.validate_and_store("   ", "sec").await;

        assert!(matches!(result, Err(CredentialError::EmptyInput)));
        assert!(!*gateway.called.lock().unwrap());
    }

    /// A1/A5: has_credentials は保存状態を反映する。A6: logout で消える。
    #[tokio::test]
    async fn has_credentials_and_logout_reflect_store_state() {
        let store = Arc::new(InMemorySecretStore::default());
        let usecase = CredentialUseCases::new(
            Arc::new(FakeGateway::new(Behavior::Success)),
            store.clone(),
        );

        assert!(!usecase.has_credentials().unwrap());

        usecase.validate_and_store("tok", "sec").await.unwrap();
        assert!(usecase.has_credentials().unwrap());

        usecase.logout().unwrap();
        assert!(!usecase.has_credentials().unwrap());
    }
}
