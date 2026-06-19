# フロントエンド（WebView/React）アーキテクチャ

| 項目       | 内容                                                                     |
| ---------- | ------------------------------------------------------------------------ |
| ステータス | Active（設計合意済み・Epic A 実装前）                                    |
| 作成日     | 2026-06-02                                                               |
| 対象       | Tauri v2 の WebView 側（React + TypeScript）                             |
| スタイル   | feature-based（[bulletproof-react](https://github.com/alan2207/bulletproof-react) 準拠） |
| 関連       | `backend.md` / `../epics/a-onboarding-auth/requirements.md`              |

## 1. 目的・方針

- **feature = Epic = バックエンドの usecase** で縦に切り、バックエンドと鏡像の構造にする。
- 依存は一方向のみ。feature 同士は依存しない。
- WebView は非特権（バックエンド設計の信頼境界の外）。資格情報そのものには一切触れない。Rust との接点は `invoke` / `listen` の2本だけで、その境界は `lib/ipc` が一手に引き受ける。
- 見た目は **Liquid Glass（グラスモーフィズム）・ダーク固定**。出典の CodePen UI kit からデザイントークンと部品を抽出し、`index.css`（トークン＋グラスCSS）＋ `components/ui`（自作の軽量グラス部品）に落とす。shadcn/ui は不採用（グラスは CSS 主導で被せる旨味が薄く、依存も増えるため。複雑な a11y プリミティブが要る時だけ Radix を単体採用）。

## 2. ディレクトリ構成（bulletproof-react 準拠）

```
src/
├─ app/                       アプリ層（ルーティングとプロバイダ）
│   ├─ routes/                  ルートコンポーネント（ページ）＝ feature の合成層
│   │   ├─ onboarding.tsx         /onboarding（credential を使用）
│   │   └─ home.tsx               /（仮置き。将来 telemetry + device を合成）
│   ├─ router.tsx               ルート定義 + ガード（RequireCredentials）
│   ├─ provider.tsx             QueryClientProvider 等
│   └─ index.tsx                App エントリ（RouterProvider を mount）
├─ features/                  機能単位（Epic に対応）
│   └─ credential/              ★Epic A
│       ├─ api.ts                 invoke 呼び出し（コマンド文字列の所有者）
│       ├─ hooks.ts               useHasCredentials / useSaveCredentials / useDeleteCredentials
│       ├─ components/
│       │   └─ OnboardingForm.tsx Token/Secret 入力フォーム
│       └─ index.ts               公開API（外に見せるものだけ re-export）
├─ components/                共有UIコンポーネント
│   └─ ui/                      自作グラスUI部品（GlassBackground/Card/Button/Input/Field/Badge/Switch）
├─ hooks/                     共有 hooks（機能横断のもののみ）
├─ lib/                       基盤・ユーティリティ
│   ├─ ipc/
│   │   ├─ invoke.ts            型付き invoke ラッパ（エラーを CommandError に正規化）
│   │   └─ types.ts             CommandError / ErrorCode（Rust 側と対の型）
│   └─ utils.ts                 cn()（依存ゼロの軽量実装。tailwind-merge は使わない）
└─ main.tsx                   エントリ（app を mount するだけ）
```

## 3. 依存ルール（一方向）

```
app（routes 含む） → features → components / hooks / lib
```

- **feature は他の feature を import しない**。共有したくなったら components / hooks / lib に降ろす。
- **components / hooks / lib は features を知らない**。
- feature の内部には `index.ts` 経由でのみアクセスする（深い import 禁止）。
- 複数 feature の合成は **`app/routes/` のルートコンポーネントだけ**が行う（例: 将来のダッシュボード = telemetry + device）。

## 4. 層の責務

| 層            | 役割                                                         | ひとことで       |
| ------------- | ------------------------------------------------------------ | ---------------- |
| `app/routes/` | feature を組み合わせてページにする（薄く。ロジック禁止）     | 「合成と配置」   |
| `app/`        | ルーター配線・プロバイダ・ガード                             | 「骨格」         |
| `features/`   | 機能の本体（UI部品・hooks・invoke 呼び出し）                 | 「機能の所有者」 |
| `components/` | 機能横断の UI 部品（自作グラス部品）                         | 「共有部品」     |
| `hooks/`      | 機能横断の hooks                                             | 「共有ロジック」 |
| `lib/`        | 基盤（ipc・ユーティリティ）。Rust との境界はここ             | 「土台」         |

> ルートコンポーネントが太り始めたら、ロジックを feature に降ろすサイン（バックエンドの「commands は薄く」と同じ規律）。

## 5. ルーティング

- **React Router v7（ライブラリモード）+ `createMemoryRouter`**。デスクトップアプリで URL バーが無いため、history API に載せる意味がなく、`tauri://` スキーマとの相性問題も避けられる。
- ルート構成:

| パス           | ルートコンポーネント | 状態                                   |
| -------------- | -------------------- | -------------------------------------- |
| `/onboarding`  | `onboarding.tsx`     | Epic A。資格情報あり → `/` へ redirect |
| `/`            | `home.tsx`           | 仮置き。将来ダッシュボード             |
| `/devices`     | （将来）             | Epic B〜E（デバイス状態/操作/シーン）  |
| `/automations` | （将来）             | Backlog（自動化）                      |
| `/settings`    | （将来）             | Backlog（資格情報の更新）。削除は実装済 |

- **ガード**: `RequireCredentials` が `useHasCredentials()`（TanStack Query）を参照。
  - `true` → `<Outlet/>`、`false` → `<Navigate to="/onboarding"/>`、ロード中 → スプラッシュ。
  - 保存/削除の mutation 成功時に `has_credentials` クエリを invalidate → ガードが自動で反応して遷移する（**画面遷移を手で書かない**）。

## 6. サーバ状態管理（TanStack Query）

- invoke の結果は「サーバ状態」（真実は Rust 側）。`useState` で手管理せず Query/Mutation に載せる。
- Epic A のマッピング:

| hook                    | 種別     | invoke               | 備考                                   |
| ----------------------- | -------- | -------------------- | -------------------------------------- |
| `useHasCredentials()`   | query    | `has_credentials`    | ガードと振り分けの根拠                 |
| `useSaveCredentials()`  | mutation | `save_credentials`   | 成功時 `has_credentials` を invalidate |
| `useDeleteCredentials()`| mutation | `delete_credentials` | 同上（将来 /settings から使用）        |

- 将来の listen（Backlog の可視化: `device-status-updated` 等）は `lib/ipc/events.ts` に型付きラッパを追加し、feature の hooks が購読して Query キャッシュへ書き込む。

## 7. IPC 境界とエラー処理

- `lib/ipc/invoke.ts` が Tauri の `invoke` をラップし、reject（`unknown`）を `CommandError` に正規化する。**invoke を生で呼ぶのは lib/ipc だけ**。
- Rust 側 `commands/error.rs` と対の型:

```typescript
export type ErrorCode =
  | "empty_input" | "unauthorized" | "rate_limited"
  | "network" | "storage" | "unexpected";

export interface CommandError {
  code: ErrorCode;   // 分岐に使う（unauthorized → 再認証へ 等）
  message: string;   // 表示にそのまま使う（Rust 側で日本語済み）
}
```

- **文言の真実は Rust 側**。フロントは `code` で分岐し、`message` を表示するだけ。
- invoke のコマンド文字列は各 feature の `api.ts` が所有する（散らばらせない）。

## 8. Epic ↔ feature 対応

| Epic | feature | バックエンドの対応 |
| --- | --- | --- |
| **A 認証** | `features/credential` | `usecases/credential.rs`（実装済） |
| **B デバイス状態** | `features/device`（一覧・状態表示） | `usecases/device`（取得系） |
| **C デバイス基本操作** | `features/device`（ON/OFF 等） | `usecases/device`（共通コマンド） |
| **D デバイス個別操作** | `features/device`（機種別UI） | `usecases/device`（機種別コマンド） |
| **E シーン** | `features/scene` | `usecases/scene` |

### Backlog（feature 化しない＝当面やらない）

センサー可視化（`features/telemetry`）・自動化（`features/automation`）・メニューバー常駐（Rust 側 tray）・設定画面（資格情報の更新）。

## 9. 技術スタック

| 領域           | 採用                                                            |
| -------------- | --------------------------------------------------------------- |
| UI             | React 19 + TypeScript + Vite                                    |
| ルーティング   | React Router v7（ライブラリモード・createMemoryRouter）         |
| サーバ状態     | TanStack Query                                                  |
| スタイリング   | Tailwind CSS v4 + 自作グラス部品（Liquid Glass・ダーク固定。`components/ui` / `lib/utils` 規約。shadcn/ui は不採用） |
| フォント       | 英語・日本語ともヒラギノ角ゴ（Hiragino Kaku Gothic）で統一。無い環境は sans-serif にフォールバック。OS同梱フォント前提で同梱なし |
| グラフ（将来） | Recharts（Backlog: 可視化）                                     |
| グローバル状態 | 採用しない（必要になってから検討）                              |
| パスエイリアス | `@/` → `src/`（tsconfig + vite に設定）                         |

## 10. 命名・コード規約

- feature 名は単数（`credential` / `device`）— バックエンドの `usecases/credential.rs` と鏡像。
- ルートコンポーネントのファイル名はパスに対応（`app/routes/onboarding.tsx` → `/onboarding`）。
- hooks は `useXxx`。mutation は `useSaveCredentials` のように動詞を含める。
- `api.ts` の関数名はバックエンドの usecase メソッドに対応（`saveCredentials` → `save_credentials` → `CredentialUseCases::save`）。

## 11. 実装状況（2026-06-19 時点）

- 完了: 依存追加（react-router / @tanstack/react-query / tailwind v4）、`lib/ipc`（invoke ラッパ＋型）、`app/`（provider / router / index 骨格）。
- 完了: **デザイン土台**＝ `index.css` の Liquid Glass トークン＋グラスCSS（ダーク固定）、`components/ui` の自作グラス部品（GlassBackground/Card/Button/Input/Field/Badge/Switch）、`lib/utils.ts` の `cn()`、フォント（英語=Consolas / 日本語=ヒラギノ角ゴ）。`home.tsx` は当面その動作確認ギャラリー（仮）。
- 次: `features/credential`（api/hooks/OnboardingForm）→ `app/router` に `/onboarding` ＋ `RequireCredentials` ガード → 雛形デモUI削除（Rust 側 `greet` コマンドも削除）。
