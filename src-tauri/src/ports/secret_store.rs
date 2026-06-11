use thiserror::Error;

use crate::models::Credentials;

/// 機密（Token/Secret）の保管庫（抽象 = port）。
///
/// 具象実装は `adapters/secret`（OSキーチェーン）に置く。
/// keyring は同期APIなので、このtraitも同期メソッドにしている。
pub trait SecretStore: Send + Sync {
    /// 資格情報を保存（既存があれば上書き）。
    fn save(&self, creds: &Credentials) -> Result<(), SecretStoreError>;

    /// 保存済みの資格情報を取得する。未保存なら `Ok(None)`。
    fn load(&self) -> Result<Option<Credentials>, SecretStoreError>;

    /// 資格情報を削除する（ログアウト）。
    fn delete(&self) -> Result<(), SecretStoreError>;
}

/// SecretStore 由来のエラー。
#[derive(Debug, Error)]
pub enum SecretStoreError {
    /// キーチェーンへの読み書きに失敗（OSが拒否した等）。
    #[error("キーチェーンへのアクセスに失敗しました: {0}")]
    Access(String),

    /// 保存データの形式が壊れている（JSONとして読めない等）。
    #[error("保存データの形式が不正です: {0}")]
    Decode(String),
}
