import type { InputHTMLAttributes } from "react";
import { forwardRef } from "react";

import { cn } from "@/lib/utils";

export interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  /** 入力エラー時 true。`aria-invalid` を立て、赤縁スタイルになる */
  invalid?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(function Input({ invalid, className, ...props }, ref) {
  return <input ref={ref} aria-invalid={invalid || undefined} className={cn("glass-input", className)} {...props} />;
});
