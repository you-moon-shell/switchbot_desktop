import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

export interface CardProps {
  /** 上部の小さなラベル（大文字・字間広め） */
  label?: string;
  /** 見出し（display フォント） */
  title?: ReactNode;
  className?: string;
  children?: ReactNode;
}

/**
 * グラスカード。背景のブロブが面の向こうに透ける。
 * 本文テキストは `.glass-card__body` を付けると半透明の落ち着いた見た目になる。
 */
export function Card({ label, title, className, children }: CardProps) {
  return (
    <div className={cn("glass glass-card", className)}>
      {/* 反射(::before/::after)より前面に内容を出す */}
      <div className="relative z-[3]">
        {label && <div className="glass-card__label">{label}</div>}
        {title && <h3 className="glass-card__title">{title}</h3>}
        {children}
      </div>
    </div>
  );
}
