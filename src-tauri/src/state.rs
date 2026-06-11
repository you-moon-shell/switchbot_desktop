use crate::usecases::CredentialUseCases;

/// アプリ全体の共有状態。
///
/// 起動時（lib.rs）に DI 済みの usecase を詰め、`tauri::Builder::manage` で登録する。
/// 各 command は `tauri::State<AppState>` 経由でここへアクセスする。
pub struct AppState {
    pub credential: CredentialUseCases,
}
