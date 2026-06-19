---
paths:
  - "src/**/*.{ts,tsx,css}"
---

# フロントエンド（React / WebView）コーディングルール

根拠・詳細は [frontend spec](../../docs/spec/architecture/frontend.md)。

## UI 要件

- **レスポンシブにしない**：デスクトップの固定ウィンドウ前提。ブレークポイント（`sm:` / `md:` 等）・可変レイアウトは入れない。
- **見た目**：Liquid Glass（グラスモーフィズム）・ダーク固定、フォントはヒラギノ角ゴ。トークン/部品は `index.css` ＋ `components/ui`。shadcn/ui は不採用。

## 構造・依存

- 依存は一方向：`app（routes 含む）→ features → components / hooks / lib`。
- **feature は他の feature を import しない**。feature 内へは `index.ts` 経由のみ（深い import 禁止）。複数 feature の合成は `app/routes/` だけ。
- **invoke を生で呼ぶのは `lib/ipc` だけ**。コマンド文字列は各 feature の `api.ts` が所有する。

## 状態・遷移

- invoke の結果は**サーバ状態**＝TanStack Query で扱う（`useState` で手管理しない）。
- 画面遷移は宣言的に：mutation 成功で関連クエリを **invalidate** し、ガード/ルートに反応させる（`navigate()` を撒かない）。
