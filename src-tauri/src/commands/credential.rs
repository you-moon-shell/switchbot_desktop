//! 認証まわりの invoke 受け口（要件 Epic A）。
//!
//! commands 層は「受付」: 入力を受け取り usecase に委譲し、エラーを CommandError に翻訳するだけ。

use tauri::State;

use super::error::{CommandError, ErrorCode};
use crate::state::AppState;
use crate::usecases::CredentialError;

/// オンボーディング: 入力された資格情報を検証し、成功時のみ保存する（A3/A4）。
#[tauri::command]
pub async fn save_credentials(
    state: State<'_, AppState>,
    token: String,
    secret: String,
) -> Result<(), CommandError> {
    state
        .credential
        .save(&token, &secret)
        .await
        .map_err(CommandError::from)
}

/// 起動時判定: 資格情報が保存済みか（A1/A5）。bool しか返さない。
#[tauri::command]
pub fn has_credentials(state: State<'_, AppState>) -> Result<bool, CommandError> {
    state.credential.exists().map_err(CommandError::from)
}

/// 資格情報を削除する（A6）。
#[tauri::command]
pub fn delete_credentials(state: State<'_, AppState>) -> Result<(), CommandError> {
    state.credential.delete().map_err(CommandError::from)
}

/// 認証ユースケースのエラー → フロント向け CommandError への翻訳。
///
/// 「どの ErrorCode に落とすか」は credential 固有の知識なので、汎用の error.rs ではなく
/// このコマンドモジュールに置く（コマンドが増えたら各 `From<XxxError>` を各モジュールへ）。
impl From<CredentialError> for CommandError {
    fn from(err: CredentialError) -> Self {
        let code = match &err {
            CredentialError::EmptyInput => ErrorCode::EmptyInput,
            CredentialError::Unauthorized => ErrorCode::Unauthorized,
            CredentialError::RateLimited => ErrorCode::RateLimited,
            CredentialError::Network(_) => ErrorCode::Network,
            CredentialError::Unexpected(_) => ErrorCode::Unexpected,
            CredentialError::Storage(_) => ErrorCode::Storage,
        };
        Self {
            code,
            message: err.to_string(),
        }
    }
}
