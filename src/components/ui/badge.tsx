import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

type BadgeColor = "aqua" | "violet" | "rose" | "amber" | "lime" | "error";

export interface BadgeProps {
  color?: BadgeColor;
  /** 先頭に脈打つドットを表示（ステータス表示用） */
  dot?: boolean;
  className?: string;
  children: ReactNode;
}

export function Badge({ color = "aqua", dot = false, className, children }: BadgeProps) {
  return (
    <span className={cn("glass-badge", `glass-badge--${color}`, className)}>
      {dot && <span className="glass-badge__dot" />}
      <span className="glass-badge__text">{children}</span>
    </span>
  );
}
