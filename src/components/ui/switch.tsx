import type { InputHTMLAttributes, ReactNode } from "react";

import { cn } from "@/lib/utils";

export interface SwitchProps
  extends Omit<InputHTMLAttributes<HTMLInputElement>, "type"> {
  /** トラックの右に出すラベル */
  label?: ReactNode;
}

/**
 * グラスのトグルスイッチ（チェックボックスを視覚的に置き換え）。
 * `checked` / `onChange` で制御する。
 */
export function Switch({ label, className, ...props }: SwitchProps) {
  return (
    <label className={cn("glass-switch-label", className)}>
      <input type="checkbox" {...props} />
      <span className="glass-switch-track">
        <span className="glass-switch-thumb" />
      </span>
      {label && <span>{label}</span>}
    </label>
  );
}
