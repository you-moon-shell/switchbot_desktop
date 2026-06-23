//! read model — 読み取り（クエリ）経路を流れる表示用の型。
//!
//! `models`（同一性・不変条件・振る舞いを持つ純粋ドメイン）とは区別する。
//! ここに置く型は「外部から読み取った結果を運ぶだけ」でドメインロジックを持たない。
//! そのため serde_json など“外形”由来の依存を持ってよい（ドメインには持ち込まない）。
pub mod device_status;

pub use device_status::DeviceStatus;
