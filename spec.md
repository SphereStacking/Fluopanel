# Arcana アプリケーション仕様書

## 概要

Arcana は macOS 用のカスタマイズ可能なデスクトップウィジェットフレームワーク。Tauri 2.0 (Rust) バックエンドと Vue 3 フロントエンドで構築。

---

## アーキテクチャ

### パッケージ構成

```
packages/
├── core/              # フレームワーク非依存のウィジェット管理コア
├── providers/         # システムデータプロバイダー (10種類)
├── vue/               # Vue 3 用ユーティリティ・コンポーネント
├── tauri/             # Tauri デスクトップアプリ (Rust バックエンド)
└── starters/
    └── vue/           # Vue 3 フロントエンド実装
```

### レイヤー構造

```
┌─────────────────────────────────────┐
│  User Widgets (Vue/React/HTML)      │  ← ユーザー定義ウィジェット
├─────────────────────────────────────┤
│  Starters (Vue)                     │  ← フレームワーク固有実装
├─────────────────────────────────────┤
│  @arcana/vue                        │  ← Vue コンポーネント・composables
├─────────────────────────────────────┤
│  @arcana/providers                  │  ← システムデータプロバイダー
├─────────────────────────────────────┤
│  @arcana/core                       │  ← WindowController, PopoverController
├─────────────────────────────────────┤
│  Tauri (Rust)                       │  ← ウィンドウ管理, IPC, macOS統合
└─────────────────────────────────────┘
```

---

## Rust バックエンド (packages/tauri/src-tauri)

### モジュール構成

```
src/
├── lib.rs              # エントリーポイント、Tauri Builder 設定
├── commands/           # Tauri IPC コマンド
│   ├── mod.rs          # コマンド登録
│   ├── aerospace.rs    # ワークスペース管理 (Aerospace CLI)
│   ├── system.rs       # システム情報 (CPU, メモリ, バッテリー等)
│   ├── icons.rs        # アプリアイコン抽出 (base64)
│   ├── config.rs       # 設定永続化
│   └── popover.rs      # ポップオーバー制御
├── windows/            # ウィンドウ管理
│   ├── mod.rs
│   ├── manager.rs      # ウィンドウ作成・配置・ライフサイクル
│   └── discovery.rs    # ウィジェットマニフェスト検出
└── ipc/                # Unix ソケット IPC
    └── mod.rs          # /tmp/arcana.sock サーバー
```

### Tauri コマンド一覧

#### ウィンドウ管理 (windows/manager.rs)
| コマンド | 説明 |
|---------|------|
| `create_inline_window` | インラインウィンドウ作成 (Window コンポーネント用) |
| `update_window_position` | ウィンドウ位置更新 |
| `hide_window` | ウィンドウ非表示 |
| `show_window` | ウィンドウ表示 |
| `close_window` | ウィンドウ閉じる |
| `get_windows` | アクティブウィンドウ一覧 |
| `create_window` | マニフェストからウィンドウ作成 |

#### ポップオーバー (commands/popover.rs)
| コマンド | 説明 |
|---------|------|
| `create_popover` | ポップオーバーパネル作成 (NSPanel) |
| `show_popover` | ポップオーバー表示 (アンカー位置指定) |
| `hide_popover` | ポップオーバー非表示 |
| `close_popover` | ポップオーバー閉じる |
| `is_popover_visible` | 表示状態確認 |
| `toggle_popover` | 表示切替 |

#### システム情報 (commands/system.rs)
| コマンド | 説明 |
|---------|------|
| `get_cpu_usage` | CPU 使用率 |
| `get_memory_info` | メモリ情報 |
| `get_battery_info` | バッテリー状態 |
| `get_network_info` | ネットワーク情報 |
| `get_volume_info` | 音量情報 |
| `set_volume` | 音量設定 |
| `get_media_info` | メディア再生情報 |
| `media_control` | メディア制御 (再生/停止等) |
| `get_disk_info` | ディスク情報 |
| `get_active_app` | アクティブアプリ情報 |
| `get_datetime` | 日時情報 |

#### Aerospace (commands/aerospace.rs)
| コマンド | 説明 |
|---------|------|
| `get_workspaces` | ワークスペース一覧 |
| `get_focused_workspace` | フォーカス中ワークスペース |
| `focus_workspace` | ワークスペース切替 |
| `get_workspace_windows` | ワークスペース内ウィンドウ |

#### その他
| コマンド | 説明 |
|---------|------|
| `get_app_icon` | アプリアイコン取得 (base64) |
| `load_config` / `save_config` | 設定読み書き |
| `discover_windows` | ウィジェット検出 |

### macOS 固有機能

#### アクセサリアプリモード (lib.rs)
```rust
app.set_activation_policy(tauri::ActivationPolicy::Accessory);
```
- Dock に表示されない
- メニューバーなし
- 他アプリがアクティブでもウィンドウ表示可能

#### NSPanel (popover.rs)
- `tauri-nspanel` クレートでポップオーバー実装
- フォーカスを奪わずに表示
- クリック外で自動非表示 (blur ハンドラー)

#### ウィンドウ位置計算 (manager.rs)
- CSS 風の位置指定: top, bottom, left, right, width, height
- モニター座標系への変換
- 論理ピクセル対応 (scale factor)

### IPC サーバー (ipc/mod.rs)

```bash
# Unix ソケット
/tmp/arcana.sock

# プロトコル
focus-changed:{workspace}:{monitor}
```

- Aerospace の `exec-on-workspace-change` から呼び出し
- イベントをフロントエンドに転送

---

## TypeScript Core (packages/core)

### エクスポート

```typescript
// ウィンドウ管理
export { WindowController } from './window-controller'
export { PopoverController } from './popover-controller'
export { ArcanaWindow } from './window'
export { WindowManager } from './window-manager'

// 型定義
export type { WindowPosition, WindowConfig, ... } from './types'
```

### WindowController

インラインウィンドウの作成・管理を担当。

```typescript
class WindowController {
  async create(config: InlineWindowConfig): Promise<void>
  async updatePosition(label: string, position: WindowPosition): Promise<void>
  async show(label: string): Promise<void>
  async hide(label: string): Promise<void>
  async close(label: string): Promise<void>
  getWindows(): Promise<string[]>
}
```

### PopoverController

ポップオーバーパネルの作成・表示制御。

```typescript
class PopoverController {
  async create(id: string, url: string, config?: PopoverConfig): Promise<void>
  async show(id: string, anchor: AnchorPosition): Promise<void>
  async hide(id: string): Promise<void>
  async toggle(id: string, anchor: AnchorPosition): Promise<void>
  async isVisible(id: string): Promise<boolean>
}
```

### WindowPosition 型

```typescript
interface WindowPosition {
  monitor?: string      // "primary" or monitor name
  top?: number
  bottom?: number
  left?: number
  right?: number
  width?: number
  height?: number
}
```

- 水平: (left + right) or (left + width) or (right + width)
- 垂直: (top + bottom) or (top + height) or (bottom + height)

---

## Vue パッケージ (packages/vue)

### コンポーネント

#### `<Window>`
インラインウィンドウを宣言的に作成。

```vue
<Window
  id="bar"
  :position="{ top: 0, left: 0, right: 0, height: 32 }"
  :transparent="true"
  :always-on-top="true"
>
  <template #content>
    <!-- ウィンドウ内コンテンツ -->
  </template>
</Window>
```

#### `<Popover>`
ポップオーバーパネルを宣言的に作成。

```vue
<Popover
  id="notifications"
  :width="400"
  :height="500"
>
  <template #content>
    <!-- ポップオーバー内コンテンツ -->
  </template>
</Popover>
```

### Composables

#### `usePopover(id)`
ポップオーバー制御フック。

```typescript
const { show, hide, toggle, isVisible } = usePopover('my-popover')

// アンカー位置を指定して表示
show({ x: 100, y: 50 })
toggle({ x: 100, y: 50 })
```

#### `useWindowMode()`
現在のウィンドウモード判定。

```typescript
const { isCoordinator, isWindow, windowLabel } = useWindowMode()
```

### ウィンドウレジストリ

```typescript
// 登録
registerWindow(id: string)

// 解除
unregisterWindow(id: string)

// 確認
isWindowRegistered(id: string): boolean
```

---

## Vue Starter (packages/starters/vue)

### アプリケーション構造

```
src/
├── App.vue              # ルートコンポーネント (モード分岐)
├── components/
│   ├── Bar.vue          # メニューバー型ウィジェット
│   ├── GitHub.vue       # GitHub 統合コンポーネント
│   ├── TestPopover.vue  # テスト用ポップオーバー
│   └── popovers/        # ポップオーバーコンテンツ
│       ├── GitHubIssuesPopover.vue
│       ├── GitHubNotificationsPopover.vue
│       ├── GitHubPRsPopover.vue
│       └── TestPopoverContent.vue
└── composables/
    ├── useArcanaInit.ts # 初期化フック
    └── useSharedStore.ts # クロスウィンドウ状態共有
```

### 2つのウィンドウモード

#### 1. Coordinator モード
- メインウィンドウ (非表示)
- 他ウィンドウの親
- `<Window>` コンポーネントをレンダリング

```vue
<!-- App.vue -->
<template v-if="isCoordinator">
  <Window id="bar" ...>
    <template #content><Bar /></template>
  </Window>

  <Popover id="github-prs" ...>
    <template #content><GitHubPRsPopover /></template>
  </Popover>
</template>
```

#### 2. Window モード
- 子ウィンドウ (inline-window-*)
- URL パラメータで ID 取得
- コンテンツのみレンダリング

```vue
<template v-else>
  <component :is="windowComponent" />
</template>
```

### SharedStore

クロスウィンドウ状態共有 (localStorage + events)。

```typescript
const store = useSharedStore()

// 状態更新 (全ウィンドウに伝播)
store.set('key', value)

// 状態取得
const value = store.get('key')

// リアクティブ監視
watch(() => store.get('key'), (newVal) => { ... })
```

---

## データフロー

### プロバイダーパターン

```typescript
interface Provider<T> {
  get(): Promise<T>
  subscribe(callback: (data: T) => void): () => void
}
```

### 利用可能なプロバイダー

| プロバイダー | データ |
|-------------|--------|
| `aerospace` | ワークスペース情報 |
| `battery` | バッテリー状態 |
| `cpu` | CPU 使用率 |
| `memory` | メモリ使用状況 |
| `network` | ネットワーク情報 |
| `date` | 日時 |
| `media` | 再生中メディア |
| `volume` | 音量 |
| `activeApp` | アクティブアプリ |
| `disk` | ディスク使用状況 |

### イベントフロー

```
┌─────────────┐    IPC Event    ┌─────────────┐
│   Rust      │ ─────────────→  │  Provider   │
│  (system)   │                 │ (subscribe) │
└─────────────┘                 └──────┬──────┘
                                       │ callback
                                       ▼
                                ┌─────────────┐
                                │ Vue Component│
                                │ (reactive)   │
                                └─────────────┘
```

---

## ウィンドウライフサイクル

### 作成フロー

1. Coordinator がマウント
2. `<Window>` コンポーネントがレンダリング
3. `WindowController.create()` → Tauri `create_inline_window`
4. Rust: `WebviewWindowBuilder` でウィンドウ作成
5. URL: `http://localhost:1420/?window={id}`
6. 子ウィンドウが Window モードでロード
7. コンテンツコンポーネントをレンダリング

### ポップオーバーフロー

1. `<Popover>` がマウント時に `PopoverController.create()`
2. Rust: NSPanel を作成 (非表示)
3. トリガー要素クリック → `show(anchor)`
4. Rust: NSPanel をアンカー位置に表示
5. 外側クリック → blur イベント → 自動非表示

---

## 設定ファイル

### 場所
```
~/.config/arcana/
├── config.json           # グローバル設定
└── widgets/              # ユーザー定義ウィジェット
    └── my-widget/
        ├── widget.json   # マニフェスト
        └── index.html    # エントリー
```

### カスタムプロトコル
```
arcana://window/{window_id}/{entry}
```

---

## 開発コマンド

```bash
npm run dev        # 全サービス並列起動
npm run dev:tauri  # Tauri のみ
npm run dev:vue    # Vue のみ
npm run build      # プロダクションビルド
```
