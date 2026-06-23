import { AlertTriangle, RefreshCw } from "lucide-react";

import { Badge, Card } from "@/components/ui";

import type { Device } from "../api";
import { useDeviceStatus } from "../hooks";

const FIELD_UNITS: Record<string, string> = {
  temperature: "°C",
  humidity: "%",
};

function formatValue(key: string, value: unknown): string {
  if (typeof value === "number") return `${value}${FIELD_UNITS[key] ?? ""}`;
  if (typeof value === "string") return value;
  return JSON.stringify(value);
}

/** 選択中デバイスの詳細（右ペイン）。物理は状態を表示、IR は状態なしを表示。 */
export function DeviceDetailPanel({ device }: { device: Device }) {
  return device.kind === "remote" ? <RemoteDetail device={device} /> : <PhysicalDetail device={device} />;
}

/** 物理デバイス：状態を取得して表示（選択中のみ取得・ポーリング）。 */
function PhysicalDetail({ device }: { device: Device }) {
  const status = useDeviceStatus(device.deviceId, { enabled: true });

  return (
    <Card label={device.deviceType}>
      <div className="mt-2 flex items-center gap-2">
        {device.hubDeviceId ? <Badge color="aqua">Hub 配下</Badge> : <Badge color="violet">直接接続</Badge>}
        <button
          type="button"
          className="icon-btn ml-auto"
          onClick={() => status.refetch()}
          disabled={status.isFetching}
          aria-label="更新"
          title="更新"
        >
          <RefreshCw size={16} className={status.isFetching ? "animate-spin" : undefined} />
        </button>
      </div>

      <div className="mt-4">
        {status.isPending && <p className="glass-card__body">状態を取得中…</p>}

        {status.isError && (
          <p className="glass-field-error" role="alert">
            <AlertTriangle size={14} aria-hidden="true" />
            {status.error.message}
          </p>
        )}

        {status.data &&
          (Object.keys(status.data.status).length === 0 ? (
            <p className="glass-card__body">表示できる状態がありません</p>
          ) : (
            <dl className="flex flex-col gap-2">
              {Object.entries(status.data.status).map(([key, value]) => (
                <div key={key} className="flex items-baseline justify-between gap-4">
                  <dt className="text-sm" style={{ color: "var(--color-text-muted)" }}>
                    {key}
                  </dt>
                  <dd className="text-sm tabular-nums">{formatValue(key, value)}</dd>
                </div>
              ))}
            </dl>
          ))}
      </div>
    </Card>
  );
}

/** 赤外線リモコン：状態を持たないので取得しない。 */
function RemoteDetail({ device }: { device: Device }) {
  return (
    <Card label={device.deviceType}>
      <div className="mt-2 flex items-center gap-2">
        <Badge color="amber">リモコン</Badge>
        {device.hubDeviceId && <Badge color="aqua">Hub 配下</Badge>}
      </div>
      <p className="glass-card__body mt-4">赤外線リモコン（状態は取得できません）。操作は今後対応予定です。</p>
    </Card>
  );
}
