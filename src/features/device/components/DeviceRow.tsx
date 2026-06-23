import { Badge } from "@/components/ui";
import { cn } from "@/lib/utils";

import type { Device } from "../api";
import { useDeviceStatus } from "../hooks";
import { iconForDevice } from "./device-icon";

/** 行のステータス: 情報なし→null / offline→"offline" / 電源off→"off" / 他→"on"。 */
function deriveRowState(fields: Record<string, unknown> | undefined): "on" | "off" | "offline" | null {
  if (fields?.onlineStatus === undefined && fields?.power === undefined) return null;
  if (fields?.onlineStatus === "offline") return "offline";
  if (fields?.power === "off") return "off";
  return "on";
}

/** 一覧の1行。物理は状態を取得し ON/OFF/Offline を表示（非ポーリング・IR は取得しない）。 */
export function DeviceRow({ device, selected, onSelect }: { device: Device; selected: boolean; onSelect: () => void }) {
  const Icon = iconForDevice(device);
  const status = useDeviceStatus(device.deviceId, {
    enabled: device.kind === "physical",
    poll: false,
  });

  const state = deriveRowState(status.data?.status);

  return (
    <button
      type="button"
      onClick={onSelect}
      aria-pressed={selected}
      className={cn("device-row", selected && "device-row--selected")}
    >
      <span className="device-row__icon" aria-hidden="true">
        <Icon size={18} />
      </span>
      <span className="min-w-0 flex-1">
        <span className="device-row__name block truncate">{device.deviceName}</span>
        <span className="device-row__sub block truncate">{device.deviceType}</span>
      </span>

      <span className="device-row__status">
        {state === "on" && (
          <Badge color="success" dot="effect">
            ON
          </Badge>
        )}
        {state === "off" && (
          <Badge color="neutral" dot="dot">
            OFF
          </Badge>
        )}
        {state === "offline" && (
          <Badge color="danger" dot="dot">
            Offline
          </Badge>
        )}
      </span>
    </button>
  );
}
