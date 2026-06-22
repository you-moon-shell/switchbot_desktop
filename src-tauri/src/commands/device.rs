//! デバイス状態の invoke 受け口（要件 Epic B）。
//!
//! commands 層は「受付」: usecase に委譲し、ドメインモデルを **Serialize 可能な DTO** に
//! 翻訳して返す（backend spec §12）。ドメイン（`models::Device`）の変更がフロントを直接
//! 壊さないようにするための境界（`CommandError` の正常系版）。

use serde::Serialize;
use serde_json::{Map, Value};
use tauri::State;

use super::error::{CommandError, ErrorCode};
use crate::models::{Device, DeviceKind, DeviceStatus};
use crate::state::AppState;
use crate::usecases::DeviceError;

/// 区分のフロント向け表現。ドメインの `DeviceKind` を生で返さず DTO で翻訳する
/// （`ErrorCode` と同じ方針）。serde で snake_case 文字列（"physical" / "remote"）になる。
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKindDto {
    Physical,
    Remote,
}

impl From<DeviceKind> for DeviceKindDto {
    fn from(kind: DeviceKind) -> Self {
        match kind {
            DeviceKind::Physical => Self::Physical,
            DeviceKind::Remote => Self::Remote,
        }
    }
}

/// 一覧表示用の DTO。フィールド名は TS 側に合わせて camelCase で出す。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDto {
    pub device_id: String,
    pub device_name: String,
    /// 物理 / IR の区別。フロントはこれを見て status を取りに行くか決める。
    pub kind: DeviceKindDto,
    pub device_type: String,
    pub hub_device_id: Option<String>,
}

impl From<Device> for DeviceDto {
    fn from(d: Device) -> Self {
        Self {
            device_id: d.device_id,
            device_name: d.device_name,
            kind: d.kind.into(),
            device_type: d.device_type,
            hub_device_id: d.hub_device_id,
        }
    }
}

/// 状態表示用の DTO。種別別フィールドは素通し（`status` にそのまま入れる）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatusDto {
    pub device_id: String,
    pub device_type: String,
    pub status: Map<String, Value>,
}

impl From<DeviceStatus> for DeviceStatusDto {
    fn from(s: DeviceStatus) -> Self {
        Self {
            device_id: s.device_id,
            device_type: s.device_type,
            status: s.fields,
        }
    }
}

/// 物理デバイス一覧を取得する（B1）。
#[tauri::command]
pub async fn list_devices(state: State<'_, AppState>) -> Result<Vec<DeviceDto>, CommandError> {
    let devices = state
        .device
        .list_devices()
        .await
        .map_err(CommandError::from)?;
    Ok(devices.into_iter().map(DeviceDto::from).collect())
}

/// 指定デバイスの現在状態を取得する（B2）。
#[tauri::command]
pub async fn get_device_status(
    state: State<'_, AppState>,
    device_id: String,
) -> Result<DeviceStatusDto, CommandError> {
    let status = state
        .device
        .get_status(&device_id)
        .await
        .map_err(CommandError::from)?;
    Ok(status.into())
}

/// デバイスユースケースのエラー → フロント向け CommandError への翻訳。
///
/// どの ErrorCode に落とすかは device 固有の知識なので、汎用の error.rs ではなくこの
/// コマンドモジュールに置く（credential.rs と同じ方針）。
impl From<DeviceError> for CommandError {
    fn from(err: DeviceError) -> Self {
        let code = match &err {
            DeviceError::Unauthorized => ErrorCode::Unauthorized,
            DeviceError::RateLimited => ErrorCode::RateLimited,
            DeviceError::Network(_) => ErrorCode::Network,
            DeviceError::Unexpected(_) => ErrorCode::Unexpected,
            DeviceError::Storage(_) => ErrorCode::Storage,
        };
        Self {
            code,
            message: err.to_string(),
        }
    }
}
