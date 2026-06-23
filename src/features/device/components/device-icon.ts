import {
  AirVent,
  Blinds,
  Box,
  Cuboid,
  Fan,
  LampFloor,
  Lightbulb,
  Lock,
  type LucideIcon,
  Mouse,
  Plug,
  Radio,
  Router,
  Thermometer,
  Tv,
} from "lucide-react";

import type { Device } from "../api";

/** 種別ラベルの部分一致で表示アイコンを選ぶ（該当なしは物理=Box / IR=Radio）。 */
export function iconForDevice(device: Device): LucideIcon {
  const t = device.deviceType.toLowerCase();
  if (t.includes("meter") || t.includes("sensor")) return Thermometer;
  if (t.includes("plug")) return Plug;
  if (t.includes("remote")) return Mouse;
  if (t.includes("button") || t.includes("bot")) return Cuboid;
  if (t.includes("hub")) return Router;
  if (t.includes("floor") || t.includes("lamp")) return LampFloor; // フロアライト
  if (t.includes("light") || t.includes("bulb")) return Lightbulb;
  if (t.includes("lock")) return Lock;
  if (t.includes("curtain") || t.includes("blind")) return Blinds;
  if (t.includes("tv")) return Tv;
  if (t.includes("air")) return AirVent; // Air Conditioner / Air Purifier
  if (t.includes("fan")) return Fan;
  return device.kind === "remote" ? Radio : Box;
}
