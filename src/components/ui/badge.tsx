import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

import { type Color, resolveColor } from "./colors";

/** ドットの種類: 脈動+発光 / 静的 / 無し。 */
export type BadgeDot = "effect" | "dot" | "none";

export interface BadgeProps {
  /** 色名（aqua 等）でも状態（success 等）でも指定できる。 */
  color?: Color;
  /** 先頭ドットの種類（既定: none）。 */
  dot?: BadgeDot;
  className?: string;
  children: ReactNode;
}

export function Badge({ color = "neutral", dot = "none", className, children }: BadgeProps) {
  return (
    <span className={cn("glass-badge", `glass-badge--${resolveColor(color)}`, className)}>
      {dot !== "none" && <span className={cn("glass-badge__dot", dot === "effect" && "glass-badge__dot--effect")} />}
      <span className="glass-badge__text">{children}</span>
    </span>
  );
}
