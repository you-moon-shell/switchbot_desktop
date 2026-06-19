import { forwardRef } from "react";
import type { ButtonHTMLAttributes } from "react";

import { cn } from "@/lib/utils";

type Variant = "primary" | "ghost" | "accent" | "danger";
type Size = "sm" | "md" | "lg";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  size?: Size;
  /** 横幅いっぱいに広げる */
  block?: boolean;
}

const variantClass: Record<Variant, string> = {
  primary: "glass-btn--primary",
  ghost: "glass-btn--ghost",
  accent: "glass-btn--accent",
  danger: "glass-btn--danger",
};

const sizeClass: Record<Size, string> = {
  sm: "glass-btn--sm",
  md: "",
  lg: "glass-btn--lg",
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { variant = "primary", size = "md", block = false, type = "button", className, ...props },
  ref,
) {
  return (
    <button
      ref={ref}
      type={type}
      className={cn(
        "glass-btn",
        variantClass[variant],
        sizeClass[size],
        block && "glass-btn--block",
        className,
      )}
      {...props}
    />
  );
});
