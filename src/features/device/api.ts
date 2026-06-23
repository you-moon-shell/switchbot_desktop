import { ipcInvoke } from "@/lib/ipc";

/**
 * デバイス状態まわりの invoke 呼び出し（Epic B）。
 * コマンド文字列（Rust 側 `commands::device::*`）はこの feature が所有し、外に散らさない。
 */

/** 物理 / 赤外線リモコンの区分（Rust 側 `DeviceKindDto`）。 */
export type DeviceKind = "physical" | "remote";

/** Rust 側 `DeviceDto`（serde camelCase）と対の型。 */
export interface Device {
  deviceId: string;
  deviceName: string;
  /** "physical"（状態あり）か "remote"（IR・状態なし）。status を取りに行くかの判断に使う。 */
  kind: DeviceKind;
  /** 種別ラベル。物理は deviceType（Bot/Meter…）、IR は remoteType（TV/Air Conditioner…）。 */
  deviceType: string;
  /** Hub 配下でなければ null（Rust の `Option<String>` → `string | null`）。 */
  hubDeviceId: string | null;
}

/** Rust 側 `DeviceStatusDto` と対の型。 */
export interface DeviceStatus {
  deviceId: string;
  deviceType: string;
  /**
   * 種別別の状態フィールド（素通し）。Rust 側で型を固定していない（`Map<String, Value>`）ため、
   * フロントでも `unknown` 値の連想配列として受け、表示側でフォールバックする（要件 §6）。
   */
  status: Record<string, unknown>;
}

/** 物理デバイス一覧を取得（B1）。失敗時は CommandError を throw。 */
export function listDevices(): Promise<Device[]> {
  return ipcInvoke<Device[]>("list_devices");
}

/**
 * 指定デバイスの現在状態を取得（B2）。
 * Tauri v2 は JS 側の camelCase キーを Rust の snake_case 引数（`device_id`）に対応づける。
 */
export function getDeviceStatus(deviceId: string): Promise<DeviceStatus> {
  return ipcInvoke<DeviceStatus>("get_device_status", { deviceId });
}
