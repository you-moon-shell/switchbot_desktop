use serde::Serialize;

/// フロントへ返すエラー（JSON にシリアライズされて invoke の reject に渡る）。
///
/// - `code`: フロントが分岐に使う（例: unauthorized なら再認証画面へ）
/// - `message`: ユーザーに表示する説明文（各層の Display を利用）
///
/// 全コマンド共通の「箱」。各コマンド固有の usecase エラー → CommandError への変換
/// （`From` 実装）は、その変換知識を持つコマンドモジュール側に置く（例: `commands/credential.rs`）。
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
    Storage,
    Unexpected,
}
