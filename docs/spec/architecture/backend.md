# バックエンド（Rustコア）アーキテクチャ

| 項目       | 内容                                                                    |
| ---------- | ----------------------------------------------------------------------- |
| ステータス | Active（**Epic A のバックエンド実装完了・テスト済み**。§13 参照）       |
| 作成日     | 2026-06-02 / 最終更新 2026-06-19                                        |
| 対象       | Tauri v2 の Rustコア（バックエンド）。フロント(WebView)は文脈として記載 |
| スタイル   | 依存性逆転（trait境界）＋ 責務別構成（repositories / gateways）         |
| 関連       | `../epics/a-onboarding-auth/requirements.md` / `frontend.md`            |

## 1. 目的・方針

SwitchBotデスクトップアプリの Rustコアを、**外界（HTTP API・SQLite・OSキーチェーン）への依存を抽象(trait)の裏に隠し、差し替え可能にする**設計でつくる。これにより、

- **テスト容易性**：実トークン・実機・ネットワーク・実キーチェーン無しで、ユースケースのロジックを検証できる（偽のGateway/Storeを差し込む）。
- **境界の明確さ**：機密・署名・通信はRustコアに閉じ込め、WebViewへは出さない。
- **拡張のしやすさ**：Epic A→B→C…と機能を足すとき、土台（repositories / gateways）を再利用できる。

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

## 3. アーキテクチャスタイル：依存性逆転 ＋ 責務別構成

サービス（ユースケース）は**抽象(trait)にだけ依存**し、具象がその trait を実装する。依存は常に内側（ユースケース/モデル）を向く（**依存性逆転**）。

旧構成は「抽象＝`ports/` ／ 具象＝`adapters/`」と**レイヤー**で分けていた。本構成では**外界との接続を責務で束ねる**：

- `repositories/` … データの**永続化**（OSキーチェーン／将来のSQLite）
- `gateways/` … **外部API**（SwitchBotクラウド）への窓口

各責務フォルダの中に **trait（契約）と具象実装を別ファイルで同居**させ、ファイル名で区別する（`*_gateway.rs`＝契約 ／ `*_api_gateway.rs`＝具象 など）。usecases が依存するのは中の trait だけで、具象は知らない——この一点（依存性逆転）は旧構成から不変。

```
        ┌──────────── usecases（ユースケース層）────────────┐
        │ CredentialUseCases / DeviceUseCases / ...          │
        │ ※ trait にだけ依存。実装の中身は知らない           │
        └──────────▲────────────────────────▲───────────────┘
                   │ depends on (trait)       │ depends on (trait)
        ┌──────────┴──────────┐   ┌───────────┴──────────────┐
        │ gateways/switchbot/ │   │ repositories/secret/      │
        │  trait SwitchBot    │   │  trait SecretRepository   │ ← 契約(trait)
        │       Gateway       │   │  ───────────────────────  │
        │  ─────────────────  │   │  KeyringSecret            │ ← 具象(impl)
        │  SwitchBotApiGateway│   │       Repository          │   同じフォルダに同居
        │  (reqwest + HMAC)   │   │  (OSキーチェーン)         │
        └──────────┬──────────┘   └───────────┬──────────────┘
                   ▼                           ▼
            SwitchBot Cloud           OSキーチェーン（将来: SQLite）
```

**依存の向き**：`commands → usecases → trait ←（同居）── 具象実装`（`models` は全層が参照する純粋な型）。trait と具象が同じフォルダにあっても、矢印は「usecases → trait」のままで具象へは向かない。

## 4. ディレクトリ構成

凡例: ✅ = 実装済み（Epic A）、⬜ = 将来実装（Epic B〜E ＝ デバイス/シーン、または Backlog）

各責務フォルダは **`<役割>.rs`＝契約(trait)** と **`<技術>_<役割>.rs`＝具象(impl)** を同居させる。

```
src-tauri/src/
├─ models/                    ドメインの型（エンティティ/値オブジェクト）
│   ├─ credential.rs          ✅ Credentials（Debug は秘密を *** に伏せる独自実装）
│   ├─ device.rs              ⬜ Device
│   ├─ telemetry.rs           ⬜ Telemetry
│   └─ automation.rs          ⬜ AutomationRule
├─ usecases/                  ユースケース（trait にだけ依存）
│   ├─ credential.rs          ✅ CredentialUseCases + CredentialError（Fakeによる単体テスト5本付き）
│   ├─ device.rs              ⬜ DeviceUseCases
│   ├─ telemetry.rs           ⬜ TelemetryUseCases
│   └─ automation.rs          ⬜ AutomationUseCases
├─ gateways/                  外部APIへの窓口（契約 trait と具象を同居）
│   ├─ switchbot/             ✅ SwitchBot クラウドAPI
│   │   ├─ switchbot_gateway.rs      契約: trait SwitchBotGateway + GatewayError
│   │   ├─ switchbot_api_gateway.rs  具象: reqwest + 型付きレスポンス(ApiResponse)
│   │   ├─ signature.rs              HMAC署名（純粋計算を分離し公式サンプル一致テスト付き）
│   │   └─ constants.rs              BASE_URL（pub(super)）
│   └─ events/                ⬜ EventPublisher（フロント通知の窓口。TauriEventPublisher。§12）
├─ repositories/              永続化（契約 trait と具象を同居）
│   ├─ secret/                ✅ 機密（OSキーチェーン）
│   │   ├─ secret_repository.rs          契約: trait SecretRepository + SecretRepositoryError（同期。keyringが同期APIのため）
│   │   ├─ keyring_secret_repository.rs  具象: JSON1エントリ保存。NoEntry→Ok(None)/delete冪等
│   │   └─ constants.rs                  SERVICE / ACCOUNT（pub(super)）
│   ├─ device/                ⬜ DeviceRepository（trait + sqlx 実装）
│   ├─ telemetry/             ⬜ TelemetryRepository（trait + sqlx 実装）
│   ├─ automation/            ⬜ AutomationRepository（trait + sqlx 実装）
│   └─ setting/               ⬜ SettingRepository（trait + sqlx 実装）
├─ commands/                  invoke受け口（usecases を呼ぶ。薄く委譲するだけ）
│   ├─ credential.rs          ✅ save_credentials / has_credentials / delete_credentials ＋ From<CredentialError> for CommandError（credential固有の翻訳）
│   └─ error.rs               ✅ CommandError { code, message } / ErrorCode（Serialize、全コマンド共通の汎用型）
├─ state.rs                   ✅ AppState（DI済み usecase の入れ物。manage で登録）
├─ lib.rs                     ✅ Composition Root（DI配線）+ Builder 起動
├─ clock.rs                   ⬜ trait Clock + 実装（時刻の抽象。永続化でもAPIでもない能力。Backlog（自動化）で必要なら。§12）
├─ poller.rs                  ⬜ 【単一】定期取得 → DB保存 → イベント発行（駆動アダプタ）
└─ scheduler.rs               ⬜ tokio-cron-scheduler（時刻/条件トリガ。駆動アダプタ）
```

## 5. レイヤーの責務

| 層             | 役割                                          | ひとことで             |
| -------------- | --------------------------------------------- | ---------------------- |
| `models`       | エンティティ/値オブジェクト（純粋な型）       | 「データの形」         |
| `usecases`     | アプリの操作ロジック（trait にだけ依存）      | 「頭脳」               |
| `gateways`     | 外部API(HTTP)への窓口：契約(trait)＋具象      | 「外界とのつなぎ(API)」 |
| `repositories` | 永続化(keychain/SQLite)：契約(trait)＋具象    | 「外界とのつなぎ(保存)」 |
| `commands`     | invoke の受け口（usecases を呼ぶ）            | 「受付」               |
| `poller`       | 単一の状態取得役                              | 「唯一の見張り番」     |
| `scheduler`    | 自動化の実行                                  | 「タイマー」           |
| `events`       | フロントへの通知（`gateways/events`）         | 「拡声器」             |

### パターンの呼び分け（意図的に別語）

- **Gateway**：外部API（SwitchBotクラウド）への窓口。`gateways/` に置く。
- **Repository**：永続化の総称。`repositories/` に置く。SQLite（履歴/ルール/設定）も、OSキーチェーンの機密（`SecretRepository`）も、同じ「保存役」として Repository に統一する。
  - ※ 旧構成では機密を `SecretStore` と呼び Repository とあえて別語にしていたが、「保存役」という責務は同じため Repository に寄せた。「機密は普通のデータと別物」という区別は、フォルダ名ではなく**専用の trait `SecretRepository`**（同期API・JSON1エントリ・キーチェーン）と専用の型・エラーで表現する。

## 6. 実装メモ（DI / エラー設計 / 署名）

### 6.1 依存性注入（実装済み）

- ユースケースは具象ではなく `Arc<dyn SwitchBotGateway>` のように **trait オブジェクト**を保持する。本番は `SwitchBotApiGateway`、テストは Fake を差し込む。
- trait の非同期メソッドは `#[async_trait]` クレートで `dyn` 対応にする。
- **`lib.rs` の `run()` が Composition Root**：手動で具象（gateway / repository）を生成し usecase に注入する（Rust では自動DIコンテナを使わず手動配線が正攻法。配線ミスはコンパイルエラーで検出される）。
- 組み立て済みの usecase は `AppState` に格納し `tauri::Builder::manage()` で登録。各 command は引数 `tauri::State<'_, AppState>` で受け取る。
- 具象型（`KeyringSecretRepository` / `SwitchBotApiGateway`）の名前が登場するのは lib.rs の配線部だけ。

### 6.2 エラー設計（実装済み）

各層が自分のエラー型を持ち、**境界で翻訳しながら**外側へ運ぶ。

```
reqwest::Error                          （実装詳細。具象の外に出さない）
  │ map_err で文字列化（手動翻訳）
  ▼
GatewayError::Network("...")            gateway の型（Unauthorized/RateLimited/Network/Unexpected）
  │ From<GatewayError>（usecaseが自層の語彙へ平坦化）
  ▼
CredentialError::Network("...")         usecase層の型に平坦化
  │                                       （EmptyInput/Unauthorized/RateLimited/Network/Unexpected/Storage）
  │ From<CredentialError>（commandで翻訳。GatewayError は見ない）
  ▼
CommandError { code, message }          Serialize → JSON → フロントの invoke().catch(e)
```

**原則**：

- 呼び出し側が**分岐に使う**情報は enum バリアントに昇格（例: `Unauthorized` → 再認証導線、`RateLimited` → ポーリング緩和）。**人間が読むだけ**の詳細は `String` に格納。
- 実装詳細のエラー型（reqwest / keyring）は具象実装で翻訳し、trait の外に漏らさない。
- **層をまたいだエラー型も境界で翻訳する**：usecase は `GatewayError` を `CredentialError` の自層バリアントに**平坦化**して保持する（`Gateway(GatewayError)` のように内側の型を抱えない）。これで `commands` は2つ下の `GatewayError` を知らずに `CredentialError` だけで `ErrorCode` に分類でき、層の独立が保てる。翻訳コストとして同義メッセージが一部重複するが、これは「漏れない境界」と引き換えの妥当なコスト。
- 詳細トレースはエラー値に背負わせず、ログ（将来 `tracing` 導入）で別途残す。その際 Token/Secret はマスキング（`Credentials` の Debug 独自実装で担保）。

#### usecase エラー細分化の基準（どこまで variant を割るか）

下層（gateway / repository）のエラーを usecase エラーにどう写すかは、**次の一点だけ**で決める：

> **外側（commands → クライアント）がその区別で「分岐」するか？**

| 判定 | やること | 例 |
| --- | --- | --- |
| 外側が分岐する | **昇格＝平坦化**：自層の variant に翻訳して持つ（下層の型は抱えない） | `GatewayError::{Unauthorized,RateLimited,Network,Unexpected}` → `CredentialError` の同名 variant（→ 別々の `ErrorCode`） |
| 外側が分岐しない（1コードに畳む / 表示のみ） | **透過保持**：下層エラーを `#[error(transparent)]` でネストのまま持つ | `SecretRepositoryError` → `CredentialError::Storage`（→ 常に `ErrorCode::storage`） |

補足の判断軸（迷ったらこの2つ）：

- **越えない線**：エラーが表すのは「**何が起きたか（ドメインの事実）**」だけ。「どう見せるか」（modal / redirect / HTTP status / i18n key 等）は**持たせない**——それは presenter（commands）の関心。
- **不変条件**：usecase は外側の型名（`ErrorCode` / `CommandError`）を**参照しない**。知識は内→外の一方向。これさえ守れば、粒度が消費者にとって有用でも「外側を知っている」ことにはならない（粒度の有用性 ≠ 型の依存）。
- **判定の検算**：その区別は GUI 無しの消費者（CLI / 単体テスト）にも意味があるか？ あるならドメインの事実なので昇格してよい（実際テストは `matches!(.., CredentialError::Network(_))` で分岐している）。

### 6.3 SwitchBot API v1.1 署名仕様（実装済み・公式サンプル一致テスト付き）

```
sign = upper( base64( HMAC-SHA256( key = secret, msg = token + t + nonce ) ) )
```

- `t` = 13桁ミリ秒タイムスタンプ / `nonce` = リクエスト毎の UUID v4 / **ボディは署名に含めない**
- 送信ヘッダ: `Authorization`(token) / `sign` / `t` / `nonce` / `Content-Type: application/json; charset=utf8`
- Base URL: `https://api.switch-bot.com/v1.1`
- 実装は `gateways/switchbot/signature.rs`。時刻・乱数を含まない純粋計算 `compute_sign` を分離し、公式READMEのPythonサンプルと同一入力での期待値一致を単体テストで担保。
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
① 起動/認証   起動 → SecretRepository からロード → 無ければOnboarding
              → SwitchBotGateway で /devices 検証 → 成功時のみ保存
② 操作        UI invoke → command → DeviceUseCase → SwitchBotGateway(署名) → 送信 → 結果返却
③ 可視化      poller(定期) → TelemetryRepository へ保存 → events.emit → UIグラフ更新
④ 自動化      scheduler/poller → 条件成立 → SwitchBotGateway でコマンド/シーン実行
```

ポイントは③の **単一ポーラー**：UIも自動化も「見張り番1人」の取得結果を共有することでコール数を抑える。

## 9. Epic ↔ モジュール対応

| Epic | 主に触るモジュール | 状態 |
| --- | --- | --- |
| **A 認証** | `commands(credential)` `usecases/credential` `gateways/switchbot` `repositories/secret` | 実装済 |
| **B デバイス状態** | `commands(device)` `usecases/device`（取得系） `gateways/switchbot`（`GET /devices`・`/status`） | 次 |
| **C デバイス基本操作** | `usecases/device`（共通コマンド ON/OFF） `gateways/switchbot`（`POST /commands`） | B の後 |
| **D デバイス個別操作** | `usecases/device`（機種別コマンド） `gateways/switchbot` | C の後 |
| **E シーン** | `commands(scene)` `usecases/scene` `gateways/switchbot`（`GET /scenes`・`execute`） | D の後 |

> **Epic A を最初にやる理由**：A は `switchbot_gateway`＋署名＋`secret_repository`＋invoke/listen の土台。B 以降はこの上に乗る。**B（読み取り）→ C（共通の書き込み）→ D（機種別）→ E（シーン）**とリスク・作業量が段階的に増える順。

### Backlog（Epic 化しない＝当面やらない）

| 項目 | 関連モジュール（将来） |
| --- | --- |
| センサー可視化 | `poller` `usecases/telemetry` `repositories/telemetry` `gateways/events(EventPublisher)` |
| 自動化 | `scheduler` `usecases/automation` `repositories/automation` `clock` `poller(条件)` |
| メニューバー常駐 | `lib.rs`（tray / ウィンドウ生存管理） |
| 設定画面（資格情報の更新） | `commands(credential)` |
| 横断 | レート制限の作り込み・`tracing` ログ・自動更新・Windows 対応 |

## 10. 技術スタック（バックエンド関連）

| 領域                | 採用                                                                    |
| ------------------- | ----------------------------------------------------------------------- |
| 枠組み              | Tauri v2（Rustコア + WebView）                                          |
| HTTP/署名           | reqwest(timeout 10s) + hmac + sha2 + base64 + uuid + serde / serde_json |
| 抽象(trait)の非同期 | async-trait                                                             |
| 永続化              | sqlx（SQLite）※未導入（Backlog: 可視化で導入）                          |
| 機密                | keyring v3（features: apple-native / windows-native）                   |
| スケジューラ        | tokio-cron-scheduler ※未導入（Backlog: 自動化で導入）                   |
| エラー              | thiserror（層ごとのエラー型を境界で翻訳。フロントへは `CommandError`）  |
| テスト              | cargo test（`#[tokio::test]` 用に dev-dependencies へ tokio macros/rt） |

## 11. 命名・コード規約

- ファイル名は型名の snake_case（例：`switchbot_gateway.rs` → `SwitchBotGateway`）。1ファイル=1 trait/型を基本。
- **契約 trait と具象は同じ責務フォルダに別ファイルで同居**する（`gateways/<x>/` `repositories/<x>/`）。契約は役割名（`*_gateway.rs` / `*_repository.rs`）、具象は「実装技術名 + 役割名」（`*_api_gateway.rs` / `keyring_*_repository.rs`）。
- `usecases/` 配下はディレクトリが役割を示すため、ファイルは領域名のみ（`usecases/credential.rs` → `CredentialUseCases`）。
- `repositories/secret/` は単数 "secret" で統一（契約 `secret_repository.rs` と具象 `keyring_secret_repository.rs` が同居）。
- モデルのファイル名も単数（`models/credential.rs`。型名は `Credentials` のまま）。
- 定数はモジュール内の `constants.rs` に切り出す（`const.rs` は予約語のため不可）。公開範囲は `pub(super)` で親モジュール（責務フォルダ内）に限定し、実装詳細を漏らさない。
- **trait の命名**：アーキテクチャ上の役割の契約は役割名詞（`Gateway` / `Repository`）。汎用能力を表す trait を作る場合は能力風の命名（標準ライブラリの `Clone` / `Iterator` などに倣う）。
- 具象は「実装技術名 + 役割trait名」（`KeyringSecretRepository`, `SwitchBotApiGateway`）。契約 trait と具象が名前だけで区別できる。

## 12. 将来の拡張方針：追加予定の抽象と「非採用」の決定

原則は一本：**「コアが外界に触れたくなったら、それが何であれ（API・DB・通知・時刻）trait を切り、責務フォルダ（gateways / repositories）に契約＋具象を同居させる。層は増やさない」**。

### 追加予定の抽象（必要になった Epic で導入）

| 抽象 | 置き場所 | 導入時期 | 理由 |
| --- | --- | --- | --- |
| `trait EventPublisher` | `gateways/events/` | **Backlog**（可視化） | ポーラー後の「フロントへ通知」も外界（WebViewへの送信窓口）。usecase が Tauri の `emit` を直接呼ぶとコアが Tauri に依存してしまうため、通知も trait + 具象（`gateways/events/` = Tauri emit 実装）のペアにする。テストでは InMemory 実装で「通知されたか」を検証できる |
| `trait Clock` | `clock.rs`（トップレベル） | **Backlog**（自動化・必要なら） | 自動化のクールダウン/ヒステリシスは時刻で分岐するロジック。`SystemTime::now()` をコアで直接呼ぶと「30分後」をテストできない。FakeClock で時間を進めて検証する。永続化でもAPI窓口でもない「能力」の抽象なので、repositories/gateways の2分類には入れず小さなトップレベル module に置く。※署名の `t` は外界との約束なので具象内の実時刻のままで良い——**コアのロジックが時刻で分岐するときだけ** trait 化する |

### 駆動側の trait を作らない（決定済み）

`CredentialUseCases` 等の**公開メソッドそのものが駆動側の入口**。駆動側の trait が活きるのは呼び出し側を差し替えたいとき（CLI版とGUI版の共用等）だが、本アプリの呼び出し側は commands 一本。被駆動側（API/キーチェーン側）だけ trait 化する非対称は意図的（テストで差し替えたいのはそちらだから）。

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

**Epic A のバックエンドが完了**（models → gateways / repositories（契約 trait ＋ 具象）→ usecases → commands → lib.rs 配線）。

- 公開 invoke コマンド：`save_credentials(token, secret)` / `has_credentials()` / `delete_credentials()`（＋雛形デモの `greet`。フロント実装時に削除予定）
- `has_credentials` は bool のみ返す＝資格情報そのものをフロントへ渡す経路を作らない。
- テスト 6本グリーン：
  - 署名が公式 Python サンプルと一致（`signature.rs`）
  - CredentialUseCases のテスト×5（AC-1: 検証成功で保存・トリム / AC-2: 401は保存しない / AC-5: ネット断は401と区別 / 空入力はAPI呼ばず拒否 / exists・delete の状態反映）— Fake注入により実トークン・実キーチェーン不要
- 未着手：`tracing` によるログ、Epic B〜E（デバイス状態/基本操作/個別操作/シーン）＋ Backlog の全モジュール。（フロントのオンボーディング画面は実装済み）

## 付録：Rust 初心者メモ

- **trait ＝ TypeScript の `interface`**。「こういうメソッドを持つ」という約束だけ。実装は別（`impl Trait for Type` = `class X implements Y`）。
- **`Arc<dyn Trait>`** ＝ 「その trait を実装した“何か”への共有ポインタ」。実行時に実装が決まる（型を interface で受ける感覚）。
- **ディレクトリ＝モジュール**。`repositories/secret/mod.rs` で各ファイルを `pub mod ...;` 宣言し、`repositories/mod.rs`・`lib.rs` の `pub mod repositories;` で読み込む。`pub` を付けたものだけ外から見える（`export` 相当）。
- **`Result<T, E>`** ＝ 「成功(`Ok`)か失敗(`Err`)」を値で返す。Rustに例外は無い。
