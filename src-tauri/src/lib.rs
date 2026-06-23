// アプリのモジュール（依存は内向き。外界は責務別に repositories / gateways へ集約）
pub mod commands;
pub mod gateways;
pub mod models;
pub mod read_models;
pub mod repositories;
pub mod state;
pub mod usecases;

use std::sync::Arc;

use gateways::{SwitchBotApiGateway, SwitchBotGateway};
use repositories::{KeyringSecretRepository, SecretRepository};
use state::AppState;
use usecases::{CredentialUseCases, DeviceUseCases};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // ── DI: 具象実装（gateway / repository）を生成し、抽象（trait）として usecase に注入する ──
    // gateway / secrets は複数の usecase で共有するため Arc を clone して配る
    // （Arc の clone は参照カウント+1 で中身は1つ。所有権ではなく共有を増やす操作）。
    let gateway: Arc<dyn SwitchBotGateway> = Arc::new(SwitchBotApiGateway::new());
    let secrets: Arc<dyn SecretRepository> = Arc::new(KeyringSecretRepository::new());
    let app_state = AppState {
        credential: CredentialUseCases::new(gateway.clone(), secrets.clone()),
        device: DeviceUseCases::new(gateway, secrets),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::credential::save_credential,
            commands::credential::has_credential,
            commands::credential::delete_credential,
            commands::device::list_devices,
            commands::device::get_device_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
