use serde_json::{Map, Value};

/// デバイス1件（一覧の要素）。
///
/// 物理デバイスと、家電をスマートリモコン化した赤外線リモコン（`infraredRemoteList`）の両方を表す。
/// API 上の差（`deviceList`/`infraredRemoteList`、`deviceType`/`remoteType`）は gateway が吸収し、
/// ここでは種別ラベルを `device_type` に統一して持つ。残るのは `kind`（物理 / IR）だけ——これは
/// 「IR には status を取りに行かない／リモコン表示にする」とフロントの**振る舞いが変わる**ために残す。
#[derive(Debug, Clone, PartialEq)]
pub struct Device {
    /// デバイス固有 ID（`status` / `commands` のパスに使う）。
    pub device_id: String,
    /// 表示名。
    pub device_name: String,
    /// 物理 / IR リモコンの区分（状態を取りに行くかの判断に使う）。
    pub kind: DeviceKind,
    /// 種別ラベル。物理は `deviceType`（Bot/Meter/Plug…）、IR は `remoteType`（TV/Air Conditioner…）。
    pub device_type: String,
    /// 親 Hub の deviceId。Hub 配下でなければ `None`。
    pub hub_device_id: Option<String>,
}

/// デバイスの区分。
///
/// `Physical` は `/status` で状態を読める。`Remote`（赤外線リモコン）は赤外線を一方的に送るだけで
/// 状態を持てないため、状態取得・ポーリングの対象外（操作＝コマンド送信は Epic C で扱う）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceKind {
    Physical,
    Remote,
}

/// デバイスの現在状態。
///
/// 種別ごとに項目が異なる（Bot は `power`、Meter は `temperature`/`humidity` …）ため、
/// **型を固定せず「取得できたフィールドをそのまま」保持する**。これにより未知・未対応の
/// 種別でもクラッシュせずフォールバック表示できる（要件 §6 エッジケース）。
///
/// Epic B は読み取り表示のみで、状態に対するドメインロジック（しきい値判定など）が無い。
/// よって素通し表現で十分。個別フィールドを型として扱いたくなったら（Epic C/D の操作で
/// 必要になれば）その時に型を起こす——という段階的拡張の判断。
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceStatus {
    pub device_id: String,
    pub device_type: String,
    /// 種別別の状態フィールド（`deviceId` / `deviceType` を除いた残り）。
    pub fields: Map<String, Value>,
}
