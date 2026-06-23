use serde_json::{Map, Value};

/// デバイスの現在状態（read model）。
///
/// ドメインエンティティではなく、外部 API から読み取った状態を表示用に運ぶだけの型。
/// 同一性・不変条件・振る舞いを持たないので `models`（純粋ドメイン）には置かず、ここに分離する。
/// これにより serde_json への依存もこの read model に閉じ、ドメインは JSON 表現を知らないまま保てる。
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
