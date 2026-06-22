# バックエンド（Rustコア）アーキテクチャ

| 項目       | 内容                                                              |
| ---------- | ----------------------------------------------------------------- |
| ステータス | Active（Epic A・B のバックエンド実装済み・テスト付き）            |
| 作成日     | 2026-06-02 / 更新 2026-06-22                                      |
| 対象       | Tauri v2 の Rustコア。フロント(WebView)は文脈として記載           |
| スタイル   | 依存性逆転（trait境界）＋ 責務別構成（gateways / repositories）   |
| 関連       | `../epics/` / `frontend.md`                                       |

## 1. 目的

外界（HTTP API・OSキーチェーン・将来の SQLite）への依存を **trait の裏に隠して差し替え可能**にする。狙いは3つ：

- **テスト容易性**：実トークン・実機・通信・実キーチェーン無しで usecase を検証（Fake を注入）。
- **境界の明確さ**：機密・署名・通信は Rustコアに閉じ込め、WebView へ出さない。
- **拡張性**：Epic A→B→C… で土台（gateways / repositories）を再利用する。

## 2. 全体像と依存の向き

```
WebView (React/TS) ── UI・非特権
   │ invoke(コマンド)        ▲ listen(イベント)
═══╪═════════ 信頼境界 ══════╪════
   ▼                        │
Rust Core ── 特権・tokio。通信/署名/機密/DB を集約
   │ HTTPS(署名付き)         │
   ▼                        ▼
SwitchBot Cloud API      OSキーチェーン（＋将来 SQLite）
```

フロント↔コアは `invoke` / `listen` の2本のみ。コア内の依存は常に内向き：

```
commands → usecases → trait ←(同フォルダに同居) 具象実装
                  models（純粋な型・全層が参照）
```

usecases は **trait にだけ依存**し具象を知らない（依存性逆転）。具象型名（`SwitchBotApiGateway` / `KeyringSecretRepository`）が登場するのは `lib.rs` の配線だけ。

## 3. ディレクトリと層の責務

責務フォルダは **契約(trait)と具象を別ファイルで同居**させる（`*_gateway.rs`＝契約 / `*_api_gateway.rs`＝具象）。

```
src-tauri/src/
├─ models/         ドメインの型（純粋）: Credentials / Device / DeviceKind / DeviceStatus
├─ usecases/       操作ロジック（trait にだけ依存）: credential / device
├─ gateways/
│   └─ switchbot/  trait SwitchBotGateway / SwitchBotApiGateway(reqwest+HMAC) / signature.rs / constants.rs
├─ repositories/
│   └─ secret/     trait SecretRepository / KeyringSecretRepository(キーチェーン) / constants.rs
├─ commands/       invoke 受け口（薄く委譲）: credential / device / error
├─ state.rs        AppState（DI済み usecase の入れ物。manage で登録）
└─ lib.rs          Composition Root（DI配線）＋ Builder 起動
```

> Backlog で追加予定：`gateways/events`（通知）、`repositories/{device,telemetry,automation,setting}`（SQLite）、`poller` / `scheduler` / `clock`。**層は増やさず**、外界が増えたら責務フォルダに trait＋具象を足す。

| 層             | 役割                                                       |
| -------------- | ---------------------------------------------------------- |
| `models`       | データの形（エンティティ/値オブジェクト・純粋）            |
| `usecases`     | 操作ロジック（判断・分岐・オーケストレーション。trait依存） |
| `gateways`     | 外部API への窓口（契約＋具象）                             |
| `repositories` | 永続化（keychain / SQLite。契約＋具象）                    |
| `commands`     | invoke の受付（usecases へ委譲＋DTO/エラー翻訳）           |

**Gateway と Repository の使い分け**：外部APIは Gateway、保存役は Repository（SQLite も機密キーチェーンも統一）。「機密は別物」はフォルダ名でなく、専用 trait `SecretRepository`（同期API・JSON1エントリ）と専用の型・エラーで表現する。

## 4. エラー設計

各層が自分のエラー型を持ち、**境界で翻訳**して外へ運ぶ：

```
reqwest::Error → GatewayError(Network等) → CredentialError/DeviceError → CommandError{code,message} → JSON
             map_err           From(平坦化)            From(翻訳)            Serialize → invoke().catch
```

下層エラーを usecase エラーへ写す粒度は **「外側（commands→クライアント）がその区別で分岐するか」** だけで決める：

| 外側が…       | 方針                                                       | 例                                                                 |
| ------------- | ---------------------------------------------------------- | ------------------------------------------------------------------ |
| 分岐する      | **昇格＝平坦化**（自層 variant に翻訳。下層型は抱えない）   | `GatewayError::{Unauthorized,RateLimited,…}` → 同名 variant → 別 `ErrorCode` |
| 分岐しない    | **透過保持**（`#[error(transparent)]` でネストのまま）     | `SecretRepositoryError` → `…::Storage` → 常に `ErrorCode::storage`  |

- 分岐に使う情報は enum バリアントへ、人間が読むだけの詳細は `String` に。
- エラーが表すのは「**何が起きたか（ドメインの事実）**」だけ。「どう見せるか」（modal/redirect 等）は commands の関心。
- usecase は外側の型（`ErrorCode` / `CommandError`）を参照しない（知識は内→外の一方向）。GUI 無しの消費者（CLI/テスト）にも意味がある区別なら昇格してよい。
- Token/Secret はログでマスキング（`Credentials` の Debug 独自実装で担保）。

## 5. SwitchBot API v1.1 署名

```
sign = upper( base64( HMAC-SHA256( key = secret, msg = token + t + nonce ) ) )
```

- `t`=13桁ミリ秒 / `nonce`=UUIDv4 / **ボディは含めない**。ヘッダ: `Authorization(token)` / `sign` / `t` / `nonce` / `Content-Type`。Base URL `https://api.switch-bot.com/v1.1`。
- 実装は `signature.rs`：時刻・乱数を含まない純粋計算 `compute_sign` を分離し、公式サンプルとの一致を単体テストで担保。
- HTTP 200 でも body の `statusCode != 100` は失敗扱い（型付き封筒 `ApiResponse<T>` で判定）。

## 6. DI（Composition Root）

- usecase は `Arc<dyn SwitchBotGateway>` 等の **trait オブジェクト**を保持（本番=具象 / テスト=Fake）。trait の非同期は `#[async_trait]`。
- `lib.rs` の `run()` で具象を生成し usecase に注入 → `AppState` に格納 → `manage()` 登録 → command が `State<'_, AppState>` で受け取る。手動配線（DIコンテナ不使用。配線ミスはコンパイルエラーで検出）。

## 7. 横断的な設計決定（コアの“憲法”）

| 決定                                            | 理由                              |
| ----------------------------------------------- | --------------------------------- |
| 機密・署名・HTTP は **Rustコアのみ**            | WebView に鍵を出さない            |
| 状態取得は **単一ポーラー（将来）に集約**       | レート制限（約1万/日）を守る      |
| フロント↔コアは **invoke/listen の2本だけ**     | 境界を明確に保つ                  |
| 常駐・自動化は **アプリ稼働中のみ**             | OS常駐を使わず構成を単純に        |
| 永続化 = **SQLite(履歴/ルール/設定) ＋ Keychain(機密)** | 役割分担。機密はDBに置かない |

## 8. 主要データフロー

```
① 認証     起動 → SecretRepository ロード → 無→Onboarding → /devices 検証 → 成功時のみ保存
② 表示     invoke → command → DeviceUseCases → SwitchBotGateway(署名) → 一覧/状態を返す
③ 可視化(将来) poller(定期) → TelemetryRepository 保存 → events.emit → UI更新
④ 自動化(将来) scheduler/poller → 条件成立 → SwitchBotGateway でコマンド/シーン実行
```

③④の肝は **単一ポーラー**：UIも自動化も「見張り番1人」の取得結果を共有してコール数を抑える。

## 9. Epic ↔ モジュール

| Epic           | 主に触るモジュール                                                          | 状態   |
| -------------- | --------------------------------------------------------------------------- | ------ |
| A 認証         | `commands/credential` `usecases/credential` `gateways/switchbot` `repositories/secret` | 実装済 |
| B デバイス状態 | `commands/device` `usecases/device`(取得) `gateways/switchbot`(`/devices`・`/status`)  | 実装済 |
| C 基本操作     | `usecases/device`(ON/OFF) `gateways/switchbot`(`POST /commands`)            | 次     |
| D 個別操作     | `usecases/device`(機種別コマンド)                                           | C後    |
| E シーン       | `commands/scene` `usecases/scene` `gateways/switchbot`(`/scenes`・execute)  | D後    |

B→C→D→E は **読み取り → 共通の書き込み → 機種別 → シーン** と、リスク・作業量が段階的に増える順。

**Backlog**：センサー可視化（poller / telemetry / events）、自動化（scheduler / automation / clock）、メニューバー常駐、設定画面（資格情報更新）、`tracing` ログ・Windows 対応。

## 10. 命名・コード規約

- ファイル名は型名の snake_case（1ファイル=1 trait/型）。契約は役割名（`*_gateway.rs` / `*_repository.rs`）、具象は技術名+役割（`*_api_gateway.rs` / `keyring_*_repository.rs`）。
- `usecases/` `models/` はフォルダ/領域名で役割が分かるのでファイルは領域名・単数（`credential.rs` → `CredentialUseCases` / `Credentials`）。
- 定数は各モジュールの `constants.rs`（`const.rs` は不可）。公開範囲は `pub(super)` で責務フォルダ内に限定。
- trait は役割名詞（`Gateway` / `Repository`）、汎用能力は能力名（`Clock` 等）。具象は「技術名＋役割trait名」。
- **境界型の命名**：受信ワイヤ型（gateway 内・private）は `~Body`（endpoint の body）/ `~Content`（空でない単数中身）/ `~Item`（リスト要素）/ `~List`（複数）。送信は commands の `~Dto`（`rename_all = "camelCase"`）。ドメイン型は無印（`Device` 等・serde を持たない）。

## 11. 将来の拡張・非採用

原則：**コアが外界に触れたくなったら（API・DB・通知・時刻）trait を切り、責務フォルダ（gateways / repositories）に契約＋具象を同居させる。層は増やさない**。

- **追加予定**：`EventPublisher`（`gateways/events`・フロント通知の窓口。テストは InMemory で「通知されたか」を検証）／ `Clock`（`clock.rs`・自動化の時刻分岐用。FakeClock で時間を進める。**コアのロジックが時刻で分岐する時だけ** trait 化。署名の `t` は具象内の実時刻で可）。
- **駆動側の trait は作らない**：呼び出し側は commands 一本なので不要。被駆動側（API/保存）だけ trait 化する非対称は意図的（テストで差し替えたいのはそちら）。
- **応答 DTO（Epic B〜）**：ドメイン型を生で返さず commands に Serialize 可能な DTO を定義して翻訳（ドメイン変更がフロントを直接壊さない）。

| 非採用                              | 理由                                                |
| ----------------------------------- | --------------------------------------------------- |
| ドメインサービス層                  | ロジックが薄い。必要なら `models/` に関数を足す     |
| CQRS / ドメインイベント             | 単一ユーザーのデスクトップアプリに利益なし          |
| トランザクション抽象（UnitOfWork）  | SQLite の単純書込のみ。sqlx の Tx を具象内で使えば足りる |
| DI コンテナライブラリ               | 手動配線で全依存が目で追える方が良い                |

## 12. 技術スタック（バックエンド関連）

| 領域           | 採用                                                            |
| -------------- | --------------------------------------------------------------- |
| 枠組み         | Tauri v2（Rustコア + WebView）                                  |
| HTTP/署名      | reqwest(timeout 10s) + hmac + sha2 + base64 + uuid + serde/serde_json |
| trait 非同期   | async-trait                                                     |
| 機密           | keyring v3（features: apple-native / windows-native）           |
| 永続化         | sqlx（SQLite）※Backlog（可視化）で導入                          |
| スケジューラ   | tokio-cron-scheduler ※Backlog（自動化）で導入                   |
| エラー         | thiserror（層ごとの型を境界で翻訳。フロントへは `CommandError`） |
| テスト         | cargo test（`#[tokio::test]`）                                  |

## 付録：Rust 初心者メモ

- **trait ≒ TS の `interface`**、`impl Trait for T` ≒ `class implements`。**`Arc<dyn Trait>`** = その trait を実装した“何か”への共有ポインタ（interface 型で受ける感覚・実行時に実装が決まる）。
- **`Result<T,E>`**（成功 `Ok`/失敗 `Err`・例外なし）/ **`Option<T>`**（あり/なし・`null` なし）。`?` は「`Err` は即 return / `Ok` は中身取り出し」、内部で `From` を呼ぶので `impl From<下層> for 自層` を書くと**自動翻訳**される（§4 平坦化の実体）。`opt.ok_or(e)` で `None→Err(e)`。
- **`Vec<T>`** ≒ `T[]`。**ジェネリクス `<T>`** は型ごとに実体生成（実行時に消えない）。`<T>`=静的・速い／`dyn`=実行時差し替え可：「型違いで回す」→`<T>`、「本番/テストで差し替え」→`Arc<dyn>`。
- **`Arc::clone`** = 参照カウント +1（実体は1つを共有。`gateway.clone()` で複数 usecase に配れる）。
- **serde**：`rename` / `rename_all = "camelCase"` でキー名（`deviceId`↔`device_id`）を対応、`flatten` で名前付き以外の残りを回収（`{a, ...rest}`）。
