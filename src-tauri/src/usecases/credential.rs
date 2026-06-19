use std::sync::Arc;

use thiserror::Error;

use crate::gateways::{GatewayError, SwitchBotGateway};
use crate::models::Credentials;
use crate::repositories::{SecretRepository, SecretRepositoryError};

/// 認証まわりのユースケース（要件 Epic A）。
///
/// 具象には依存せず、2つの抽象（trait）にだけ依存する。
/// 本番は SwitchBotApiGateway + KeyringSecretRepository、テストは Fake を注入する。
pub struct CredentialUseCases {
    gateway: Arc<dyn SwitchBotGateway>,
    secrets: Arc<dyn SecretRepository>,
}

/// 認証ユースケースのエラー。
///
/// gateway 由来のエラーは **この層の語彙に平坦化して**保持する（内側の `GatewayError` を
/// `Gateway(GatewayError)` のように抱えない）。こうすると外側の `commands` 層は
/// `GatewayError` を知らずに `CredentialError` だけを見て `ErrorCode` に振り分けられる
/// ——層をまたいだ深掘りマッチ（`CredentialError::Gateway(GatewayError::..)`）を避けられる。
#[derive(Debug, Error)]
pub enum CredentialError {
    /// 入力が空（トリム後）。API を呼ぶまでもなく弾く。
    #[error("トークンとシークレットを入力してください")]
    EmptyInput,

    /// トークン/シークレットが無効（API 401）。
    #[error("認証に失敗しました（トークンまたはシークレットが無効です）")]
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

    /// 保存層（キーチェーン）の操作に失敗。
    /// 外側は保存系をすべて 1 つの ErrorCode(storage) に畳むため、Gateway と違い細分化せず透過保持でよい。
    #[error(transparent)]
    Storage(#[from] SecretRepositoryError),
}

/// gateway の語彙 → usecase の語彙へ翻訳（＝境界での翻訳）。
///
/// この `From` があるおかげで `save()` 内の `?` がそのまま使え、かつ `GatewayError` が
/// `CredentialError` の外（commands）へ漏れない。
impl From<GatewayError> for CredentialError {
    fn from(err: GatewayError) -> Self {
        match err {
            GatewayError::Unauthorized => Self::Unauthorized,
            GatewayError::RateLimited => Self::RateLimited,
            GatewayError::Network(msg) => Self::Network(msg),
            GatewayError::Unexpected(msg) => Self::Unexpected(msg),
        }
    }
}

impl CredentialUseCases {
    pub fn new(gateway: Arc<dyn SwitchBotGateway>, secrets: Arc<dyn SecretRepository>) -> Self {
        Self { gateway, secrets }
    }

    /// 資格情報を検証し、成功した場合のみ保存する（要件 A3/A4）。検証なしの保存経路は持たない。
    ///
    /// - 前後の空白はトリムする（要件エッジケース）
    /// - 空入力は API を呼ばずに弾く
    /// - 検証失敗（401/ネットワーク）時は一切保存しない
    pub async fn save(&self, token: &str, secret: &str) -> Result<(), CredentialError> {
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
    pub fn exists(&self) -> Result<bool, CredentialError> {
        Ok(self.secrets.load()?.is_some())
    }

    /// 資格情報を削除する（要件 A6）。
    pub fn delete(&self) -> Result<(), CredentialError> {
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

    /// 偽の SecretRepository。キーチェーンの代わりにメモリ上の変数に保存する。
    #[derive(Default)]
    struct InMemorySecretRepository {
        saved: Mutex<Option<Credentials>>,
    }

    impl SecretRepository for InMemorySecretRepository {
        fn save(&self, creds: &Credentials) -> Result<(), SecretRepositoryError> {
            *self.saved.lock().unwrap() = Some(creds.clone());
            Ok(())
        }
        fn load(&self) -> Result<Option<Credentials>, SecretRepositoryError> {
            Ok(self.saved.lock().unwrap().clone())
        }
        fn delete(&self) -> Result<(), SecretRepositoryError> {
            *self.saved.lock().unwrap() = None;
            Ok(())
        }
    }

    /// AC-1相当: 正しい資格情報 → 保存される。前後の空白はトリムされる。
    #[tokio::test]
    async fn valid_credentials_are_trimmed_and_saved() {
        let store = Arc::new(InMemorySecretRepository::default());
        let usecase =
            CredentialUseCases::new(Arc::new(FakeGateway::new(Behavior::Success)), store.clone());

        usecase.save("  tok  ", "\tsec\n").await.unwrap();

        let saved = store.saved.lock().unwrap().clone().unwrap();
        assert_eq!(saved.token, "tok");
        assert_eq!(saved.secret, "sec");
    }

    /// AC-2: 401 → エラーになり、保存されない。
    #[tokio::test]
    async fn unauthorized_credentials_are_not_saved() {
        let store = Arc::new(InMemorySecretRepository::default());
        let usecase = CredentialUseCases::new(
            Arc::new(FakeGateway::new(Behavior::Unauthorized)),
            store.clone(),
        );

        let result = usecase.save("tok", "sec").await;

        assert!(matches!(result, Err(CredentialError::Unauthorized)));
        assert!(store.saved.lock().unwrap().is_none());
    }

    /// AC-5相当: ネットワーク断 → 401 とは別のエラーになり、保存されない。
    #[tokio::test]
    async fn network_error_is_distinguished_and_not_saved() {
        let store = Arc::new(InMemorySecretRepository::default());
        let usecase = CredentialUseCases::new(
            Arc::new(FakeGateway::new(Behavior::NetworkDown)),
            store.clone(),
        );

        let result = usecase.save("tok", "sec").await;

        assert!(matches!(result, Err(CredentialError::Network(_))));
        assert!(store.saved.lock().unwrap().is_none());
    }

    /// エッジケース: 空入力は API を呼ばずに弾く。
    #[tokio::test]
    async fn empty_input_is_rejected_without_calling_api() {
        let gateway = Arc::new(FakeGateway::new(Behavior::Success));
        let usecase = CredentialUseCases::new(
            gateway.clone(),
            Arc::new(InMemorySecretRepository::default()),
        );

        let result = usecase.save("   ", "sec").await;

        assert!(matches!(result, Err(CredentialError::EmptyInput)));
        assert!(!*gateway.called.lock().unwrap());
    }

    /// A1/A5: exists は保存状態を反映する。A6: delete で消える。
    #[tokio::test]
    async fn exists_and_delete_reflect_store_state() {
        let store = Arc::new(InMemorySecretRepository::default());
        let usecase =
            CredentialUseCases::new(Arc::new(FakeGateway::new(Behavior::Success)), store.clone());

        assert!(!usecase.exists().unwrap());

        usecase.save("tok", "sec").await.unwrap();
        assert!(usecase.exists().unwrap());

        usecase.delete().unwrap();
        assert!(!usecase.exists().unwrap());
    }
}
