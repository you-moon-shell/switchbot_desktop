use std::time::Duration;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use super::constants::BASE_URL;
use super::signature::build_signed_headers;
use super::{GatewayError, SwitchBotGateway};
use crate::models::{Credential, Device, DeviceKind, DeviceStatus};

/// SwitchBot API の共通レスポンス封筒。
///
/// 仕様: 成功時は `statusCode = 100`。`body` は呼び出すエンドポイントごとに形が変わるので
/// 型パラメータ `T` にする（一覧なら `DevicesBody`、状態なら `StatusBody`）。
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiResponse<T> {
    status_code: i64,
    body: Option<T>,
}

/// `GET /devices` の body。物理デバイス（`deviceList`）と、家電をスマートリモコン化した
/// 赤外線リモコン（`infraredRemoteList`）の2配列を持つ。両方を `Device` に正規化する。
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevicesBody {
    device_list: Vec<DeviceItem>,
    /// IR が1台も無いアカウントではキー自体が来ないこともあるため `default`（空 Vec）で受ける。
    #[serde(default)]
    infrared_remote_list: Vec<RemoteItem>,
}

/// 物理デバイスの要素。種別は `deviceType`。
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceItem {
    device_id: String,
    device_name: String,
    device_type: String,
    hub_device_id: Option<String>,
}

/// 赤外線リモコンの要素。種別は `remoteType`（物理と違い `deviceType` は無い）。
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoteItem {
    device_id: String,
    device_name: String,
    remote_type: String,
    hub_device_id: Option<String>,
}

impl DevicesBody {
    /// 2つの配列をドメインの `Device` に正規化する（物理=Physical / IR=Remote）。
    fn into_devices(self) -> Vec<Device> {
        let physical = self.device_list.into_iter().map(|item| Device {
            device_id: item.device_id,
            device_name: item.device_name,
            kind: DeviceKind::Physical,
            device_type: item.device_type,
            hub_device_id: item.hub_device_id,
        });
        let remotes = self.infrared_remote_list.into_iter().map(|item| Device {
            device_id: item.device_id,
            device_name: item.device_name,
            kind: DeviceKind::Remote,
            device_type: item.remote_type,
            hub_device_id: item.hub_device_id,
        });
        physical.chain(remotes).collect()
    }
}

/// status の body の中身（空でない形）。
///
/// `deviceId` / `deviceType` だけ名前付きで取り出し、残りの種別別フィールドは
/// `#[serde(flatten)]` で `fields` にまとめて素通しする。ここは **厳密**（必須）に保つ
/// ——状態を持つ種別は必ずこの形で返ってくる、という契約を崩さない。
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct StatusContent {
    device_id: String,
    device_type: String,
    #[serde(flatten)]
    fields: Map<String, Value>,
}

/// `GET /devices/{id}/status` の body。「中身あり（`StatusContent`）」か「空 `{}`」のどちらか。
///
/// Hub のように状態を持たない種別は空オブジェクトを返す。中身ありを緩めて吸収するのではなく、
/// **空ケースだけを別バリアントで受ける**ことで、必須項目の契約を保ったまま空 body にも耐える（要件 §6）。
/// `#[serde(untagged)]` は「先に宣言したバリアントから順に試し、最初に成功した形を採用する」
/// ——`{}` は `Full` に失敗して `Empty` に落ちる。
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum StatusBody {
    Full(StatusContent),
    Empty {},
}

impl StatusBody {
    /// body をドメインの `DeviceStatus` に変換する（IO から切り離した純粋関数）。
    /// 空ケースは `device_id` をパスの値、`device_type` を `"unknown"` で埋める。
    fn into_status(self, device_id: &str) -> DeviceStatus {
        match self {
            StatusBody::Full(c) => DeviceStatus {
                device_id: c.device_id,
                device_type: c.device_type,
                fields: c.fields,
            },
            StatusBody::Empty {} => DeviceStatus {
                device_id: device_id.to_string(),
                device_type: "unknown".to_string(),
                fields: Map::new(),
            },
        }
    }
}

/// `SwitchBotGateway` の本番実装（reqwest + HMAC-SHA256 署名）。
pub struct SwitchBotApiGateway {
    http: reqwest::Client,
}

impl SwitchBotApiGateway {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("HTTPクライアントの初期化に失敗");
        Self { http }
    }

    /// 署名付き GET の共通処理。HTTP ステータスの分類と `statusCode` 判定を一箇所に集約する。
    ///
    /// - 200 かつ `statusCode == 100` → `body` を `T` にデシリアライズして返す
    /// - 401/403 → `Unauthorized` / 429 → `RateLimited` / その他 → `Unexpected`
    async fn signed_get<T: DeserializeOwned>(
        &self,
        creds: &Credential,
        path: &str,
    ) -> Result<T, GatewayError> {
        let headers = build_signed_headers(creds);
        let resp = self
            .http
            .get(format!("{BASE_URL}{path}"))
            .header("Authorization", &headers.authorization)
            .header("sign", &headers.sign)
            .header("t", &headers.t)
            .header("nonce", &headers.nonce)
            .header("Content-Type", "application/json; charset=utf8")
            .send()
            .await
            .map_err(|e| GatewayError::Network(e.to_string()))?;

        match resp.status().as_u16() {
            200 => {
                // HTTP 200 でも body の statusCode が 100 以外なら成功ではない
                let envelope: ApiResponse<T> = resp
                    .json()
                    .await
                    .map_err(|e| GatewayError::Unexpected(e.to_string()))?;
                match envelope.status_code {
                    // body: Option<T> を Result へ。Some→Ok / None→Err。
                    // ok_or_else はエラー値をクロージャで遅延生成する（String 確保を None 時だけに抑える）。
                    100 => envelope.body.ok_or_else(|| {
                        GatewayError::Unexpected("レスポンス body が空です".to_string())
                    }),
                    code => Err(GatewayError::Unexpected(format!(
                        "APIエラー (statusCode: {code})"
                    ))),
                }
            }
            401 | 403 => Err(GatewayError::Unauthorized),
            429 => Err(GatewayError::RateLimited),
            status => Err(GatewayError::Unexpected(format!("HTTP {status}"))),
        }
    }
}

#[async_trait]
impl SwitchBotGateway for SwitchBotApiGateway {
    async fn validate_credential(&self, creds: &Credential) -> Result<(), GatewayError> {
        // /devices を署名付きで叩けて 100 が返れば資格情報は有効。中身は使わない。
        let _: DevicesBody = self.signed_get(creds, "/devices").await?;
        Ok(())
    }

    async fn list_devices(&self, creds: &Credential) -> Result<Vec<Device>, GatewayError> {
        let body: DevicesBody = self.signed_get(creds, "/devices").await?;
        Ok(body.into_devices())
    }

    async fn get_device_status(
        &self,
        creds: &Credential,
        device_id: &str,
    ) -> Result<DeviceStatus, GatewayError> {
        let body: StatusBody = self
            .signed_get(creds, &format!("/devices/{device_id}/status"))
            .await?;
        Ok(body.into_status(device_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Hub 等、status が空 body `{}` を返す種別は Empty バリアントで受ける（要件 §6）。
    #[test]
    fn empty_body_is_received_as_empty_variant() {
        let resp: StatusBody = serde_json::from_value(json!({})).unwrap();
        let status = resp.into_status("HUBID");
        assert_eq!(status.device_id, "HUBID"); // パスの値にフォールバック
        assert_eq!(status.device_type, "unknown");
        assert!(status.fields.is_empty());
    }

    /// 一覧: deviceList は Physical、infraredRemoteList は Remote に正規化する。
    #[test]
    fn device_list_body_normalizes_physical_and_remote() {
        let body: DevicesBody = serde_json::from_value(json!({
            "deviceList": [
                { "deviceId": "M1", "deviceName": "温湿度計", "deviceType": "Meter", "hubDeviceId": "HUB" }
            ],
            "infraredRemoteList": [
                { "deviceId": "R1", "deviceName": "テレビ", "remoteType": "TV", "hubDeviceId": "HUB" }
            ]
        }))
        .unwrap();

        let devices = body.into_devices();

        assert_eq!(devices.len(), 2);
        // 物理（deviceType）
        assert_eq!(devices[0].kind, DeviceKind::Physical);
        assert_eq!(devices[0].device_type, "Meter");
        // IR（remoteType をラベルに採用）
        assert_eq!(devices[1].kind, DeviceKind::Remote);
        assert_eq!(devices[1].device_type, "TV");
        assert_eq!(devices[1].device_name, "テレビ");
    }

    /// IR が無いアカウント: infraredRemoteList キーが無くてもデシリアライズできる（default）。
    #[test]
    fn device_list_body_without_infrared_key_is_ok() {
        let body: DevicesBody = serde_json::from_value(json!({
            "deviceList": [
                { "deviceId": "M1", "deviceName": "温湿度計", "deviceType": "Meter", "hubDeviceId": "HUB" }
            ]
        }))
        .unwrap();

        let devices = body.into_devices();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].kind, DeviceKind::Physical);
    }

    /// 通常の種別: Full バリアントで受け、deviceId/deviceType を抜いて残りを fields に素通しする。
    #[test]
    fn full_body_is_received_as_full_variant_and_flattens_rest() {
        let resp: StatusBody = serde_json::from_value(json!({
            "deviceId": "METERID",
            "deviceType": "Meter",
            "temperature": 25.0,
            "humidity": 50,
            "battery": 100
        }))
        .unwrap();
        let status = resp.into_status("METERID");
        assert_eq!(status.device_id, "METERID");
        assert_eq!(status.device_type, "Meter");
        assert!(!status.fields.contains_key("deviceId"));
        assert!(!status.fields.contains_key("deviceType"));
        assert!(status.fields.contains_key("temperature"));
        assert_eq!(status.fields.len(), 3);
    }
}
