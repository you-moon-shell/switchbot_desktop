import { AlertTriangle, RefreshCw } from "lucide-react";
import type { ReactNode } from "react";

import { Button } from "@/components/ui";

import type { Device } from "../api";
import { useDevices, useRefreshDevices } from "../hooks";
import { DeviceRow } from "./DeviceRow";

export interface DeviceListProps {
  /** 選択中の ID（行のハイライト用）。 */
  selectedId: string | null;
  /** 行クリックで選択（再クリックで解除）。 */
  onSelect: (device: Device | null) => void;
  /** unauthorized 時に出す再認証アクション（合成層から注入）。 */
  reauthSlot?: ReactNode;
}

/** 種別ごとにグループ化。 */
function groupByType(devices: Device[]): [string, Device[]][] {
  const groups = new Map<string, Device[]>();
  for (const d of devices) {
    const arr = groups.get(d.deviceType) ?? [];
    arr.push(d);
    groups.set(d.deviceType, arr);
  }
  return [...groups.entries()].sort((a, b) => a[0].localeCompare(b[0]));
}

/** 中央ペインの中身：一覧・種別グループ・手動更新（枠と選択 state はシェル側）。 */
export function DeviceList({ selectedId, onSelect, reauthSlot }: DeviceListProps) {
  const devices = useDevices();
  const refresh = useRefreshDevices();

  const all = devices.data ?? [];
  const groups = groupByType(all);

  return (
    <>
      <header className="list-pane__header">
        <div className="min-w-0">
          <div className="list-pane__title">デバイス</div>
          <div className="list-pane__sub">{all.length} 台</div>
        </div>
        <div className="flex-1" />
        <button
          type="button"
          className="icon-btn"
          onClick={refresh}
          disabled={devices.isFetching}
          aria-label="更新"
          title="更新"
        >
          <RefreshCw size={16} className={devices.isFetching ? "animate-spin" : undefined} />
        </button>
      </header>

      <div className="list-pane__body">
        {devices.isPending && <p className="glass-card__body p-2">読み込み中…</p>}

        {devices.isError && (
          <div className="p-2">
            <p className="glass-field-error" role="alert">
              <AlertTriangle size={14} aria-hidden="true" />
              {devices.error.message}
            </p>
            <div className="mt-4">
              {devices.error.code === "unauthorized" ? (
                reauthSlot
              ) : (
                <Button variant="ghost" onClick={refresh}>
                  再試行
                </Button>
              )}
            </div>
          </div>
        )}

        {devices.isSuccess && all.length === 0 && <p className="glass-card__body p-2">デバイスがありません</p>}

        {groups.map(([type, items]) => (
          <div key={type} className="mb-1">
            <div className="group-header">
              {type}
              <span style={{ color: "var(--color-text-subtle)" }}>{items.length}</span>
            </div>

            {items.map((device) => (
              <DeviceRow
                key={device.deviceId}
                device={device}
                selected={device.deviceId === selectedId}
                onSelect={() => onSelect(device.deviceId === selectedId ? null : device)}
              />
            ))}
          </div>
        ))}
      </div>
    </>
  );
}
