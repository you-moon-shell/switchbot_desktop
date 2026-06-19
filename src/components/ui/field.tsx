import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

export interface FieldProps {
  /** 紐づける control の id（label の htmlFor に使う） */
  id: string;
  label: string;
  /** 補足説明（error が無いときだけ表示） */
  hint?: string;
  /** エラー文（あれば赤字で表示。`role="alert"`） */
  error?: string;
  className?: string;
  /** Input / Switch などの control */
  children: ReactNode;
}

/**
 * ラベル＋control＋補足/エラーをまとめる薄いラッパ。
 * control 側の id を合わせて渡すこと（label との対応のため）。
 */
export function Field({ id, label, hint, error, className, children }: FieldProps) {
  return (
    <div className={cn("flex flex-col", className)}>
      <label htmlFor={id} className="glass-label">
        {label}
      </label>
      {children}
      {hint && !error && (
        <p
          className="mt-1"
          style={{ fontSize: "var(--text-xs)", color: "var(--color-text-subtle)" }}
        >
          {hint}
        </p>
      )}
      {error && (
        <p className="glass-field-error" role="alert">
          <span aria-hidden="true">⚠</span>
          {error}
        </p>
      )}
    </div>
  );
}
