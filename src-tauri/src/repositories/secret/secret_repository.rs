use thiserror::Error;

use crate::models::Credential;

/// 機密（Token/Secret）の保管庫（抽象 = trait）。
///
/// 具象実装は同じ `repositories/secret`（OSキーチェーン）に置く。
/// keyring は同期APIなので、このtraitも同期メソッドにしている。
pub trait SecretRepository: Send + Sync {
    /// 資格情報を保存（既存があれば上書き）。
    fn save(&self, creds: &Credential) -> Result<(), SecretRepositoryError>;

    /// 保存済みの資格情報を取得する。未保存なら `Ok(None)`。
    fn load(&self) -> Result<Option<Credential>, SecretRepositoryError>;

    /// 資格情報を削除する（ログアウト）。
    fn delete(&self) -> Result<(), SecretRepositoryError>;
}

/// SecretRepository 由来のエラー。
#[derive(Debug, Error)]
pub enum SecretRepositoryError {
    /// キーチェーンへの読み書きに失敗（OSが拒否した等）。
    #[error("キーチェーンへのアクセスに失敗しました: {0}")]
    Access(String),

    /// 保存データの形式が壊れている（JSONとして読めない等）。
    #[error("保存データの形式が不正です: {0}")]
    Decode(String),
}
