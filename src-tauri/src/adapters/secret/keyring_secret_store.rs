use keyring::Entry;

use super::constants::{ACCOUNT, SERVICE};
use crate::models::Credentials;
use crate::ports::{SecretStore, SecretStoreError};

/// `SecretStore` の OSキーチェーン実装（macOS Keychain / Windows Credential Manager）。
///
/// Credentials を JSON 文字列にして1エントリに保存する。
pub struct KeyringSecretStore;

impl KeyringSecretStore {
    pub fn new() -> Self {
        Self
    }

    /// キーチェーンのエントリ（保存場所のハンドル）を作る。
    fn entry(&self) -> Result<Entry, SecretStoreError> {
        Entry::new(SERVICE, ACCOUNT).map_err(|e| SecretStoreError::Access(e.to_string()))
    }
}

impl SecretStore for KeyringSecretStore {
    fn save(&self, creds: &Credentials) -> Result<(), SecretStoreError> {
        let json =
            serde_json::to_string(creds).map_err(|e| SecretStoreError::Decode(e.to_string()))?;
        self.entry()?
            .set_password(&json)
            .map_err(|e| SecretStoreError::Access(e.to_string()))
    }

    fn load(&self) -> Result<Option<Credentials>, SecretStoreError> {
        match self.entry()?.get_password() {
            Ok(json) => {
                let creds = serde_json::from_str(&json)
                    .map_err(|e| SecretStoreError::Decode(e.to_string()))?;
                Ok(Some(creds))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SecretStoreError::Access(e.to_string())),
        }
    }

    fn delete(&self) -> Result<(), SecretStoreError> {
        match self.entry()?.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(SecretStoreError::Access(e.to_string())),
        }
    }
}
