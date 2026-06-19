use serde::Serialize;

use crate::gateways::GatewayError;
use crate::usecases::CredentialError;

/// フロントへ返すエラー（JSON にシリアライズされて invoke の reject に渡る）。
///
/// - `code`: フロントが分岐に使う（例: unauthorized なら再認証画面へ）
/// - `message`: ユーザーに表示する説明文（各層の Display を利用）
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: ErrorCode,
    pub message: String,
}

/// フロントが分岐に使うエラー種別。
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    EmptyInput,
    Unauthorized,
    RateLimited,
    Network,
    Secret,
    Unexpected,
}

impl From<CredentialError> for CommandError {
    fn from(err: CredentialError) -> Self {
        let code = match &err {
            CredentialError::EmptyInput => ErrorCode::EmptyInput,
            CredentialError::Gateway(GatewayError::Unauthorized) => ErrorCode::Unauthorized,
            CredentialError::Gateway(GatewayError::RateLimited) => ErrorCode::RateLimited,
            CredentialError::Gateway(GatewayError::Network(_)) => ErrorCode::Network,
            CredentialError::Gateway(GatewayError::Unexpected(_)) => ErrorCode::Unexpected,
            CredentialError::Secret(_) => ErrorCode::Secret,
        };
        Self {
            code,
            message: err.to_string(),
        }
    }
}
