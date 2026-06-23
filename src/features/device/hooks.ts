import { useQuery, useQueryClient } from "@tanstack/react-query";

import type { CommandError } from "@/lib/ipc";

import { type Device, type DeviceStatus, getDeviceStatus, listDevices } from "./api";

/**
 * クエリキー。先頭を共通の "devices" にしておくと、手動更新（B3）で
 * `["devices"]` を invalidate するだけで一覧・全状態をまとめて再取得できる。
 */
const DEVICES_KEY = ["devices"] as const;
const listKey = [...DEVICES_KEY, "list"] as const;
const statusKey = (deviceId: string) => [...DEVICES_KEY, "status", deviceId] as const;

/**
 * 状態ポーリングの既定間隔（B5/B6）。
 * 「台数 × 頻度」がそのままコール数になるため、上限（約1万/日）に対して保守的に30秒から始める。
 * 既定値はコードで持ち、運用しながら調整する（要件 B6 NOTE）。
 */
const STATUS_POLL_INTERVAL_MS = 30_000;

/** 物理デバイス一覧（B1）。一覧は頻繁に変わらないので自動ポーリングはせず、手動更新に任せる。 */
export function useDevices() {
  return useQuery<Device[], CommandError>({
    queryKey: listKey,
    queryFn: listDevices,
  });
}

/**
 * 1台の現在状態（B2）。デバイスごとに独立したクエリにすることで、
 * - 1台の取得が失敗しても他のカードは表示を継続できる（AC-B5）
 * - 同じ deviceId のクエリは TanStack Query が dedup する＝取得経路が単一に集約される（AC-B6）
 *
 * ポーリング（B5）:
 * - `refetchInterval` で一定間隔の自動取得
 * - `refetchIntervalInBackground: false` で、ウィンドウ非アクティブ中はポーリング停止（バックグラウンドで叩かない）
 * - `refetchOnWindowFocus: true` で、再アクティブ時に即時1回取得（provider のグローバル false をこのクエリだけ上書き）
 */
export function useDeviceStatus(deviceId: string, options?: { enabled?: boolean; poll?: boolean }) {
  // poll=false は一覧の行（電源/オンライン表示）用。読み込み・手動更新・再フォーカス時のみ取得し、
  // 定期ポーリングはしない＝「台数 × 頻度」のコール数を抑える（B6）。
  // 定期更新（poll=true・既定）は詳細を開いた選択中の1台だけに効かせる。
  // 同じ deviceId のクエリは dedup されるので、選択中デバイスは行と詳細でキャッシュを共有する。
  const poll = options?.poll ?? true;
  return useQuery<DeviceStatus, CommandError>({
    queryKey: statusKey(deviceId),
    queryFn: () => getDeviceStatus(deviceId),
    enabled: options?.enabled ?? true,
    refetchInterval: poll ? STATUS_POLL_INTERVAL_MS : false,
    refetchIntervalInBackground: false,
    refetchOnWindowFocus: true,
  });
}

/** 手動更新（B3）。一覧と全状態をまとめて無効化＝再取得させる関数を返す。 */
export function useRefreshDevices() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: DEVICES_KEY });
}
