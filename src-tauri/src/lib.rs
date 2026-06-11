// アプリのモジュール（ヘキサゴナル構成）
pub mod adapters;
pub mod commands;
pub mod models;
pub mod ports;
pub mod state;
pub mod usecases;

use std::sync::Arc;

use adapters::secret::KeyringSecretStore;
use adapters::switchbot::SwitchBotApiGateway;
use ports::{SecretStore, SwitchBotGateway};
use state::AppState;
use usecases::AuthUseCase;

// TODO: オンボーディング画面の実装が終わったら削除する（雛形のデモコマンド）
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // ── DI: 具象 adapter を生成し、抽象（port）として usecase に注入する ──
    let gateway: Arc<dyn SwitchBotGateway> = Arc::new(SwitchBotApiGateway::new());
    let secrets: Arc<dyn SecretStore> = Arc::new(KeyringSecretStore::new());
    let app_state = AppState {
        auth: AuthUseCase::new(gateway, secrets),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::auth::save_credentials,
            commands::auth::has_credentials,
            commands::auth::logout,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
