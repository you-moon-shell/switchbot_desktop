use keyring::Entry;

use super::constants::{ACCOUNT, SERVICE};
use super::{SecretRepository, SecretRepositoryError};
use crate::models::Credentials;

/// `SecretRepository` の OSキーチェーン実装（macOS Keychain / Windows Credential Manager）。
///
/// Credentials を JSON 文字列にして1エントリに保存する。
pub struct KeyringSecretRepository;

impl KeyringSecretRepository {
    pub fn new() -> Self {
        Self
    }

    /// キーチェーンのエントリ（保存場所のハンドル）を作る。
    fn entry(&self) -> Result<Entry, SecretRepositoryError> {
        Entry::new(SERVICE, ACCOUNT).map_err(|e| SecretRepositoryError::Access(e.to_string()))
    }
}

impl SecretRepository for KeyringSecretRepository {
    fn save(&self, creds: &Credentials) -> Result<(), SecretRepositoryError> {
        let json = serde_json::to_string(creds)
            .map_err(|e| SecretRepositoryError::Decode(e.to_string()))?;
        self.entry()?
            .set_password(&json)
            .map_err(|e| SecretRepositoryError::Access(e.to_string()))
    }

    fn load(&self) -> Result<Option<Credentials>, SecretRepositoryError> {
        match self.entry()?.get_password() {
            Ok(json) => {
                let creds = serde_json::from_str(&json)
                    .map_err(|e| SecretRepositoryError::Decode(e.to_string()))?;
                Ok(Some(creds))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SecretRepositoryError::Access(e.to_string())),
        }
    }

    fn delete(&self) -> Result<(), SecretRepositoryError> {
        match self.entry()?.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(SecretRepositoryError::Access(e.to_string())),
        }
    }
}
