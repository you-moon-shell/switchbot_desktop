import type { ButtonHTMLAttributes } from "react";
import { forwardRef } from "react";

import { cn } from "@/lib/utils";

import { type Color, resolveColor } from "./colors";

/** 構造: 塗り（color が効く） / 枠線グラス（ニュートラル）。 */
type Variant = "solid" | "ghost";
type Size = "sm" | "md" | "lg";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** 塗りの色。色名（aqua 等）でも状態（danger 等）でも指定できる（variant=solid のとき有効）。 */
  color?: Color;
  variant?: Variant;
  size?: Size;
  /** 横幅いっぱいに広げる */
  block?: boolean;
}

const sizeClass: Record<Size, string> = {
  sm: "glass-btn--sm",
  md: "",
  lg: "glass-btn--lg",
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { color = "info", variant = "solid", size = "md", block = false, type = "button", className, ...props },
  ref,
) {
  // ghost は構造（ニュートラル枠）。solid は color で塗る。
  const colorClass = variant === "ghost" ? "glass-btn--ghost" : `glass-btn--${resolveColor(color)}`;

  return (
    <button
      ref={ref}
      type={type}
      className={cn("glass-btn", colorClass, sizeClass[size], block && "glass-btn--block", className)}
      {...props}
    />
  );
});
