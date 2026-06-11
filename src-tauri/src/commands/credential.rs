//! 認証まわりの invoke 受け口（要件 Epic A）。
//!
//! commands 層は「受付」: 入力を受け取り usecase に委譲し、エラーを CommandError に翻訳するだけ。

use tauri::State;

use super::error::CommandError;
use crate::state::AppState;

/// オンボーディング: 入力された資格情報を検証し、成功時のみ保存する（A3/A4）。
#[tauri::command]
pub async fn save_credentials(
    state: State<'_, AppState>,
    token: String,
    secret: String,
) -> Result<(), CommandError> {
    state
        .credential
        .validate_and_store(&token, &secret)
        .await
        .map_err(CommandError::from)
}

/// 起動時判定: 資格情報が保存済みか（A1/A5）。bool しか返さない。
#[tauri::command]
pub fn has_credentials(state: State<'_, AppState>) -> Result<bool, CommandError> {
    state.credential.has_credentials().map_err(CommandError::from)
}

/// ログアウト: 資格情報を削除する（A6）。
#[tauri::command]
pub fn logout(state: State<'_, AppState>) -> Result<(), CommandError> {
    state.credential.logout().map_err(CommandError::from)
}
