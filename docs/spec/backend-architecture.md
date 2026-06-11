# バックエンド（Rustコア）アーキテクチャ

| 項目       | 内容                                                                    |
| ---------- | ----------------------------------------------------------------------- |
| ステータス | Active（**Epic A のバックエンド実装完了・テスト済み**。§13 参照）       |
| 作成日     | 2026-06-02 / 最終更新 2026-06-02                                        |
| 対象       | Tauri v2 の Rustコア（バックエンド）。フロント(WebView)は文脈として記載 |
| スタイル   | Ports & Adapters（ヘキサゴナル / クリーンアーキテクチャ）               |
| 関連       | `docs/spec/epic-a-onboarding-auth/requirements.md`                      |

## 1. 目的・方針

SwitchBotデスクトップアプリの Rustコアを、**外界（HTTP API・SQLite・OSキーチェーン）への依存を抽象(trait)の裏に隠し、差し替え可能にする**設計でつくる。これにより、

- **テスト容易性**：実トークン・実機・ネットワーク・実キーチェーン無しで、ユースケースのロジックを検証できる（偽のGateway/Storeを差し込む）。
- **境界の明確さ**：機密・署名・通信はRustコアに閉じ込め、WebViewへは出さない。
- **拡張のしやすさ**：Epic A→B→C…と機能を足すとき、土台（ports/adapters）を再利用できる。

## 2. システム内の位置づけ（3層）

```
┌─────────────────────────────────────────────────────────────┐
│  WebView (React/TS) ── UI層・非特権                           │
│  画面: オンボーディング/ダッシュボード/デバイス/自動化/設定     │
└──────────────┬──────────────────────────▲───────────────────┘
       invoke(コマンド)│                    │listen(イベント)
══════════════╪═══════════ 信頼境界 ════════╪═══════════════════
              ▼                            │
┌─────────────────────────────────────────────────────────────┐
│  Rust Core（本ドキュメントの対象）── 特権・tokioランタイム     │
│  全API通信/署名/機密/DB/スケジューラをここに集約               │
└──────┬───────────────────────────────┬──────────────────────┘
       │HTTPS(署名付き)                 │
       ▼                               ▼
  SwitchBot Cloud API           OSキーチェーン ＋ ローカルSQLite
```

**原則**：UI(WebView)は「お願いする側」、Rust Coreが「すべての特権を持つ側」。フロント↔コアの通信は `invoke`/`listen` の2本のみ。

## 3. アーキテクチャスタイル：Ports & Adapters

サービス（ユースケース）は**抽象(port=trait)にだけ依存**し、具象(adapter)がそのportを実装する。依存は常に内側（ユースケース/モデル）を向く。

```
        ┌──────────── usecases（ユースケース層）────────────┐
        │ CredentialUseCases / DeviceUseCases / TelemetryUseCases ... │
        │ ※ ports(trait)にだけ依存。実装の中身は知らない     │
        └──────▲──────────────▲──────────────────▲──────────┘
               │ implements    │ implements       │ implements
        ┌──────┴──────┐ ┌──────┴───────┐   ┌──────┴────────┐
        │ Gateway     │ │ Repository    │   │ SecretStore   │  ← ports（抽象/trait）
        │ (外部API)   │ │ (永続化)      │   │ (機密)        │
        └──────▲──────┘ └──────▲────────┘   └──────▲────────┘
               │ impl          │ impl              │ impl
        ┌──────┴──────┐ ┌──────┴───────┐   ┌──────┴────────┐
        │SwitchBotApi │ │ Sqlite*Repo  │   │ Keyring       │  ← adapters（具象）
        │Gateway      │ │ (sqlx)       │   │ SecretStore   │
        │(reqwest+HMAC)│└──────┬───────┘   └──────┬────────┘
        └──────┬──────┘        │                  │
               ▼               ▼                  ▼
        SwitchBot Cloud      SQLite           OSキーチェーン
```

**依存の向き**：`commands → usecases → ports ← adapters`（`models` は全層が参照する純粋な型）。

## 4. ディレクトリ構成

凡例: ✅ = 実装済み（Epic A）、⬜ = 将来実装（Epic B〜F）

```
src-tauri/src/
├─ models/                    ドメインの型（エンティティ/値オブジェクト）
│   ├─ credential.rs          ✅ Credentials（Debug は秘密を *** に伏せる独自実装）
│   ├─ device.rs              ⬜ Device
│   ├─ telemetry.rs           ⬜ Telemetry
│   └─ automation.rs          ⬜ AutomationRule
├─ ports/                     抽象(trait)＝"契約"だけ
│   ├─ switchbot_gateway.rs   ✅ trait SwitchBotGateway + GatewayError
│   ├─ secret_store.rs        ✅ trait SecretStore + SecretStoreError（同期。keyringが同期APIのため）
│   ├─ device_repository.rs   ⬜ trait DeviceRepository
│   ├─ telemetry_repository.rs ⬜ trait TelemetryRepository
│   ├─ automation_repository.rs ⬜ trait AutomationRepository
│   ├─ setting_repository.rs  ⬜ trait SettingRepository
│   ├─ event_publisher.rs     ⬜ trait EventPublisher（フロント通知の抽象。§12）
│   └─ clock.rs               ⬜ trait Clock（時刻の抽象。Epic D で必要なら。§12）
├─ usecases/                  ユースケース（ports にだけ依存）
│   ├─ credential.rs          ✅ CredentialUseCases + CredentialError（Fakeによる単体テスト5本付き）
│   ├─ device.rs              ⬜ DeviceUseCases
│   ├─ telemetry.rs           ⬜ TelemetryUseCases
│   └─ automation.rs          ⬜ AutomationUseCases
├─ adapters/                  ports の実装（具象）
│   ├─ switchbot/             ✅ SwitchBotApiGateway
│   │   ├─ constants.rs            BASE_URL（pub(super)）
│   │   ├─ signature.rs            HMAC署名（純粋計算を分離し公式サンプル一致テスト付き）
│   │   └─ switchbot_api_gateway.rs reqwest + 型付きレスポンス(ApiResponse)
│   ├─ secret/                ✅ KeyringSecretStore
│   │   ├─ constants.rs            SERVICE / ACCOUNT（pub(super)）
│   │   └─ keyring_secret_store.rs JSON1エントリ保存。NoEntry→Ok(None)/delete冪等
│   ├─ persistence/           ⬜ Sqlite*Repository（sqlx）
│   └─ events/                ⬜ TauriEventPublisher（emit で WebView へ push）
├─ commands/                  invoke受け口（usecases を呼ぶ。薄く委譲するだけ）
│   ├─ credential.rs          ✅ save_credentials / has_credentials / delete_credentials
│   └─ error.rs               ✅ CommandError { code, message }（Serialize、フロント向け）
├─ state.rs                   ✅ AppState（DI済み usecase の入れ物。manage で登録）
├─ lib.rs                     ✅ Composition Root（DI配線）+ Builder 起動
├─ poller.rs                  ⬜ 【単一】定期取得 → DB保存 → イベント発行（駆動アダプタ）
└─ scheduler.rs               ⬜ tokio-cron-scheduler（時刻/条件トリガ。駆動アダプタ）
```

## 5. レイヤーの責務

| 層          | 役割                                     | ひとことで         |
| ----------- | ---------------------------------------- | ------------------ |
| `models`    | エンティティ/値オブジェクト（純粋な型）  | 「データの形」     |
| `ports`     | 抽象(trait)＝外界に求める契約            | 「約束」           |
| `usecases`  | アプリの操作ロジック（ports にだけ依存） | 「頭脳」           |
| `adapters`  | ports の具象実装（HTTP/SQLite/keyring）  | 「外界とのつなぎ」 |
| `commands`  | invoke の受け口（usecases を呼ぶ）       | 「受付」           |
| `poller`    | 単一の状態取得役                         | 「唯一の見張り番」 |
| `scheduler` | 自動化の実行                             | 「タイマー」       |
| `events`    | フロントへの通知                         | 「拡声器」         |

### パターンの呼び分け（意図的に別語）

- **Gateway**：外部API（SwitchBotクラウド）への窓口。
- **Repository**：ドメインデータの永続化（SQLite）。
- **SecretStore**：機密の保管（OSキーチェーン）。「普通のデータ(Repository)」と「秘密」を混同させないため別語。

## 6. 実装メモ（DI / エラー設計 / 署名）

### 6.1 依存性注入（実装済み）

- ユースケースは具象ではなく `Arc<dyn SwitchBotGateway>` のように **trait オブジェクト**を保持する。本番は `SwitchBotApiGateway`、テストは Fake を差し込む。
- ポート（trait）の非同期メソッドは `#[async_trait]` クレートで `dyn` 対応にする。
- **`lib.rs` の `run()` が Composition Root**：手動で adapter を生成し usecase に注入する（Rust では自動DIコンテナを使わず手動配線が正攻法。配線ミスはコンパイルエラーで検出される）。
- 組み立て済みの usecase は `AppState` に格納し `tauri::Builder::manage()` で登録。各 command は引数 `tauri::State<'_, AppState>` で受け取る。
- 具象型（`KeyringSecretStore` / `SwitchBotApiGateway`）の名前が登場するのは lib.rs の配線部だけ。

### 6.2 エラー設計（実装済み）

各層が自分のエラー型を持ち、**境界で翻訳しながら**外側へ運ぶ。

```
reqwest::Error                          （実装詳細。adapterの外に出さない）
  │ map_err で文字列化（手動翻訳）
  ▼
GatewayError::Network("...")            ports層の型（Unauthorized/RateLimited/Network/Unexpected）
  │ ? + #[from]（自動翻訳）
  ▼
CredentialError::Gateway(...)           usecase層の型（+ EmptyInput / Secret）
  │ From<CredentialError>（commandで翻訳）
  ▼
CommandError { code, message }          Serialize → JSON → フロントの invoke().catch(e)
```

**原則**：

- 呼び出し側が**分岐に使う**情報は enum バリアントに昇格（例: `Unauthorized` → 再認証導線、`RateLimited` → ポーリング緩和）。**人間が読むだけ**の詳細は `String` に格納。
- 実装詳細のエラー型（reqwest / keyring）は adapter で翻訳し、port の外に漏らさない。
- 詳細トレースはエラー値に背負わせず、ログ（将来 `tracing` 導入）で別途残す。その際 Token/Secret はマスキング（`Credentials` の Debug 独自実装で担保）。

### 6.3 SwitchBot API v1.1 署名仕様（実装済み・公式サンプル一致テスト付き）

```
sign = upper( base64( HMAC-SHA256( key = secret, msg = token + t + nonce ) ) )
```

- `t` = 13桁ミリ秒タイムスタンプ / `nonce` = リクエスト毎の UUID v4 / **ボディは署名に含めない**
- 送信ヘッダ: `Authorization`(token) / `sign` / `t` / `nonce` / `Content-Type: application/json; charset=utf8`
- Base URL: `https://api.switch-bot.com/v1.1`
- 実装は `adapters/switchbot/signature.rs`。時刻・乱数を含まない純粋計算 `compute_sign` を分離し、公式READMEのPythonサンプルと同一入力での期待値一致を単体テストで担保。
- HTTP 200 でも body の `statusCode != 100` は失敗として扱う（型付きレスポンス `ApiResponse` で判定）。

## 7. 横断的な設計決定（このコアの“憲法”）

| 決定                                                    | 理由                                      |
| ------------------------------------------------------- | ----------------------------------------- |
| 機密・署名・HTTP は **Rustコアのみ**                    | WebViewに鍵を出さない                     |
| 状態取得は **単一ポーラーに集約**                       | レート制限（約1万コール/日）を守る        |
| フロント↔コアは **invoke/listen の2本だけ**             | 境界を明確に保つ                          |
| **メニューバー常駐**・自動化は**アプリ稼働中のみ**実行  | OS常駐(launchd等)を使わず構成をシンプルに |
| 永続化 = **SQLite(履歴/ルール/設定) ＋ Keychain(機密)** | 役割分担。機密はDBに置かない              |

## 8. 主要データフロー

```
① 起動/認証   起動 → SecretStore からロード → 無ければOnboarding
              → SwitchBotGateway で /devices 検証 → 成功時のみ保存
② 操作        UI invoke → command → DeviceUseCase → SwitchBotGateway(署名) → 送信 → 結果返却
③ 可視化      poller(定期) → TelemetryRepository へ保存 → events.emit → UIグラフ更新
④ 自動化      scheduler/poller → 条件成立 → SwitchBotGateway でコマンド/シーン実行
```

ポイントは③の **単一ポーラー**：UIも自動化も「見張り番1人」の取得結果を共有することでコール数を抑える。

## 9. Epic ↔ モジュール対応

| Epic             | 主に触るモジュール                                                                                                     |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------- |
| **A 認証**       | `commands(credential)` `usecases/credential` `ports/switchbot_gateway` `ports/secret_store` `adapters/switchbot` `adapters/secret` |
| B デバイス操作   | `usecases/device` `ports/switchbot_gateway` `adapters/switchbot`                                                       |
| C センサー可視化 | `poller` `usecases/telemetry` `ports/telemetry_repository` `ports/event_publisher` `adapters/persistence` `adapters/events` |
| D 自動化         | `scheduler` `usecases/automation` `ports/automation_repository` `ports/clock(必要なら)` `poller(条件)`                 |
| E 常駐           | `lib.rs`（tray/ウィンドウ生存管理）                                                                                    |
| F 横断           | `adapters/switchbot`（レート管理） `events` エラー設計                                                                 |

> **Epic A を最初にやる理由**：A は `switchbot_gateway`＋署名＋`secret_store`＋invoke/listen の土台を作る部分。B〜F は全部この土台の上に乗る。

## 10. 技術スタック（バックエンド関連）

| 領域                | 採用                                                                    |
| ------------------- | ----------------------------------------------------------------------- |
| 枠組み              | Tauri v2（Rustコア + WebView）                                          |
| HTTP/署名           | reqwest(timeout 10s) + hmac + sha2 + base64 + uuid + serde / serde_json |
| 抽象(trait)の非同期 | async-trait                                                             |
| 永続化              | sqlx（SQLite）※未導入（Epic C で導入）                                  |
| 機密                | keyring v3（features: apple-native / windows-native）                   |
| スケジューラ        | tokio-cron-scheduler ※未導入（Epic D で導入）                           |
| エラー              | thiserror（層ごとのエラー型を境界で翻訳。フロントへは `CommandError`）  |
| テスト              | cargo test（`#[tokio::test]` 用に dev-dependencies へ tokio macros/rt） |

## 11. 命名・コード規約

- ファイル名は型名の snake_case（例：`switchbot_gateway.rs` → `SwitchBotGateway`）。1ファイル=1 trait/型を基本。
- `usecases/` 配下はディレクトリが役割を示すため、ファイルは領域名のみ（`usecases/credential.rs` → `CredentialUseCases`）。
- `adapters/secret/` と `ports/secret_store.rs` は単数 "secret" で統一。
- モデルのファイル名も単数（`models/credential.rs`。型名は `Credentials` のまま）。
- 定数はモジュール内の `constants.rs` に切り出す（`const.rs` は予約語のため不可）。公開範囲は `pub(super)` で親モジュール（アダプタ内）に限定し、実装詳細を漏らさない。
- **trait の命名**：アーキテクチャ上の役割の契約は役割名詞（`Gateway` / `Repository` / `Store`）。汎用能力を表す trait を作る場合は能力風の命名（標準ライブラリの `Clone` / `Iterator` などに倣う）。
- 具象 adapter は「実装技術名 + port名」（`KeyringSecretStore`, `SwitchBotApiGateway`）。抽象/具象が名前だけで区別できる。

## 12. 将来の拡張方針：追加予定の ports と「非採用」の決定

原則は一本：**「コアが外界に触れたくなったら、それが何であれ（API・DB・通知・時刻）port を切る。層は増やさない」**。

### 追加予定の ports（必要になった Epic で導入）

| port | 導入時期 | 理由 |
| --- | --- | --- |
| `ports/event_publisher.rs`<br>`trait EventPublisher` | **Epic C** | ポーラー後の「フロントへ通知」も外界。usecase が Tauri の `emit` を直接呼ぶとコアが Tauri に依存してしまうため、通知も port + adapter（`adapters/events/` = Tauri emit 実装）のペアにする。テストでは InMemory 実装で「通知されたか」を検証できる |
| `ports/clock.rs`<br>`trait Clock` | **Epic D**（必要なら） | 自動化のクールダウン/ヒステリシスは時刻で分岐するロジック。`SystemTime::now()` をコアで直接呼ぶと「30分後」をテストできない。FakeClock で時間を進めて検証する。※署名の `t` は外界との約束なので adapter 内の実時刻のままで良い——**コアのロジックが時刻で分岐するときだけ** port 化する |

### 駆動ポートの trait を作らない（決定済み）

`CredentialUseCases` 等の**公開メソッドそのものを駆動ポート**とする。駆動側の trait が活きるのは呼び出し側を差し替えたいとき（CLI版とGUI版の共用等）だが、本アプリの呼び出し側は commands 一本。被駆動側だけ trait 化する非対称は意図的（テストで差し替えたいのは API/キーチェーン側だから）。

### 非採用（過剰設計の防止）

| 候補 | 判定 | 理由 |
| --- | --- | --- |
| ドメインサービス層 | 不採用 | ドメインロジックが薄い。必要になれば `models/` に関数を足す |
| CQRS / ドメインイベント | 不採用 | 単一ユーザーのデスクトップアプリに読み書き分離の利益なし |
| トランザクション抽象（UnitOfWork） | 不採用 | SQLite の単純な書込のみ。sqlx のトランザクションを adapter 内で使えば足りる |
| DI コンテナライブラリ | 不採用 | 手動配線（Composition Root）で全依存が目で追える方が良い |

### commands 層の応答 DTO（Epic B から）

デバイス一覧などを返すときは、ドメインモデル（`models::Device`）をそのままフロントへ返さず、**commands 層に Serialize 可能な DTO**（例: `DeviceDto`）を定義して翻訳する。ドメインの変更がフロントを直接壊さないようにする（`CommandError` の正常系版。新しい層ではなく駆動アダプタの語彙の拡張）。

## 13. 実装状況（2026-06-02 時点）

**Epic A のバックエンドが完了**（ports → adapters → usecases → commands → lib.rs 配線）。

- 公開 invoke コマンド：`save_credentials(token, secret)` / `has_credentials()` / `delete_credentials()`（＋雛形デモの `greet`。フロント実装時に削除予定）
- `has_credentials` は bool のみ返す＝資格情報そのものをフロントへ渡す経路を作らない。
- テスト 6本グリーン：
  - 署名が公式 Python サンプルと一致（`signature.rs`）
  - CredentialUseCases のテスト×5（AC-1: 検証成功で保存・トリム / AC-2: 401は保存しない / AC-5: ネット断は401と区別 / 空入力はAPI呼ばず拒否 / exists・delete の状態反映）— Fake注入により実トークン・実キーチェーン不要
- 未着手：フロント（オンボーディング画面）、`tracing` によるログ、Epic B〜F の全モジュール。

## 付録：Rust 初心者メモ

- **trait ＝ TypeScript の `interface`**。「こういうメソッドを持つ」という約束だけ。実装は別（`impl Trait for Type` = `class X implements Y`）。
- **`Arc<dyn Trait>`** ＝ 「その trait を実装した“何か”への共有ポインタ」。実行時に実装が決まる（型を interface で受ける感覚）。
- **ディレクトリ＝モジュール**。`ports/mod.rs` で各ファイルを `pub mod ...;` 宣言し、`lib.rs` に `mod ports;` で読み込む。`pub` を付けたものだけ外から見える（`export` 相当）。
- **`Result<T, E>`** ＝ 「成功(`Ok`)か失敗(`Err`)」を値で返す。Rustに例外は無い。
