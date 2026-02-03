# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

Arcana は Tauri 2 (Rust バックエンド) で構築された macOS 用カスタマイズ可能なウィジェットフレームワークです。ユーザーは Vue/React/HTML でカスタムウィジェットを作成し、メニューバー型やフローティング型のウィジェットを画面に配置できます。

## 設計原則

### 後方互換性より Clean Architecture
- **後方互換性は不要。よりクリーンな構成を常に優先する**
- 破壊的変更を恐れず、最適な設計を追求する
- 「動いているから触らない」ではなく「より良くできるなら変える」

### フレームワーク非依存
- **Core パッケージ (`@arcana/core`) はフレームワーク非依存であること**
- フレームワーク固有のコード（ライフサイクル、リアクティビティ）は各 starter に配置
- Core には純粋な TypeScript ロジックのみ

### アーキテクチャ層
```
┌─────────────────────────────────────┐
│  Starters (Vue)                     │  ← App.vue でウィジェットを定義
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

## ビルド & 開発コマンド

```bash
# 開発 (libs watch + Tauri + Vue を並列起動)
npm run dev

# 個別起動
npm run dev:tauri     # Tauri のみ（UI は別ターミナルで起動済み前提）
npm run dev:vue       # Vue のみ（Tauri なしで UI 開発）
npm run dev:libs      # Core + Providers の watch

# プロダクションビルド (.app バンドル作成)
npm run build

# 個別ビルド
npm run build:vue     # Vue フロントエンドのみ
npm run build:tauri   # Tauri のみ
npm run build:libs    # Core + Providers

# Tauri CLI コマンドを直接実行
npm run tauri -- <command>
```

## アーキテクチャ

### モノレポ構造

```
packages/
├── core/              # WindowController, PopoverController (フレームワーク非依存)
├── providers/         # システムデータプロバイダー (10種類)
├── vue/               # Vue コンポーネント (Window, Popover) + composables
├── tauri/             # Tauri デスクトップアプリ (Rust)
│   └── src-tauri/src/
│       ├── commands/  # Tauri IPC コマンド (aerospace, system, icons, config, popover)
│       ├── windows/   # ウィンドウ管理 (manager.rs)
│       ├── ipc/       # Unix ソケット IPC
│       └── lib.rs     # エントリーポイント
└── starters/
    └── vue/           # Vue 3 フロントエンド実装
```

### Coordinator / Window モードパターン

アプリは2つのモードで動作:

1. **Coordinator モード** (メインウィンドウ、非表示)
   - `<Window>` / `<Popover>` コンポーネントをレンダリング
   - 子ウィンドウを生成・管理

2. **Window モード** (子ウィンドウ、`inline-window-*`)
   - URL パラメータ `?window={id}` で ID を取得
   - コンテンツコンポーネントのみをレンダリング

```vue
<!-- App.vue -->
<template v-if="isCoordinator">
  <Window id="bar"><template #content><Bar /></template></Window>
</template>
<template v-else>
  <component :is="windowComponent" />
</template>
```

### 設定ファイル

```
~/.config/arcana/
└── arcana.json           # グローバル設定
```

ウィジェットは `App.vue` 内で `<Window>` コンポーネントとして定義します。

### パッケージ依存関係

- `@arcana/tauri` → `@arcana/starter-vue` のビルドをフロントエンドとして実行
- `@arcana/starter-vue` → `@arcana/core` (ウィジェット管理) + `@arcana/providers` (システムデータ)
- `@arcana/core` → Tauri IPC コマンドを TypeScript インターフェースでラップ
- `@arcana/providers` → Tauri IPC コマンドをプロバイダーパターンでラップ

### プロバイダーパターン

すべてのシステムデータは `@arcana/providers` を通じて流れます:

```typescript
interface Provider<T> {
  get(): Promise<T>           // 現在のデータを取得
  subscribe(cb): () => void   // 更新を購読、購読解除関数を返す
}
```

利用可能なプロバイダー: `aerospace`, `battery`, `cpu`, `memory`, `network`, `date`, `media`, `volume`, `activeApp`, `disk`

### 命名規則

| 種類 | パターン |
|------|---------|
| 公式パッケージ | `@arcana/{name}` |
| サードパーティプロバイダー | `arcana-provider-{name}` |

### IPC レイヤー

フロントエンドは Tauri コマンドを通じて Rust と通信します:
- `commands/aerospace.rs` - ワークスペース管理 (Aerospace CLI)
- `commands/system.rs` - バッテリー、CPU、メモリ、ネットワーク、ボリューム、メディア、ディスク
- `commands/icons.rs` - アプリアイコン抽出 (base64)
- `commands/config.rs` - 設定の永続化
- `commands/popover.rs` - ポップオーバー制御 (NSPanel)
- `windows/manager.rs` - インラインウィンドウ作成・配置

### CLI & IPC

Unix ソケット (`/tmp/arcana.sock`) を使用した IPC:

```bash
arcana emit workspace-changed   # ワークスペース変更通知
```

- Aerospace の `exec-on-workspace-change` から呼び出し可能
- 既存インスタンスにイベント送信

### カスタムプロトコル

`arcana://lib/` プロトコルで共有ライブラリ（Vue, Tauri API）を配信します。

### ウィンドウ動作

- フレームレス、透明、常に最前面
- Dock から非表示（`cocoa` クレートによるアクセサリアプリ）
- モニター変更時に再配置（NSNotificationCenter オブザーバー）
- CSS absolute 風の position 指定（top, left, right, bottom, width, height）
- px, vh, vw, calc() 対応

## 主要ファイル

| 用途 | 場所 |
|------|------|
| Core 型定義 | `packages/core/src/types.ts` |
| プロバイダー型定義 | `packages/providers/src/types.ts` |
| Tauri エントリーポイント | `packages/tauri/src-tauri/src/lib.rs` |
| ウィンドウ管理 (Rust) | `packages/tauri/src-tauri/src/windows/manager.rs` |
| ポップオーバー (Rust) | `packages/tauri/src-tauri/src/commands/popover.rs` |
| IPC サーバー | `packages/tauri/src-tauri/src/ipc/mod.rs` |
| Vue Window コンポーネント | `packages/vue/src/Window.vue` |
| Vue Popover コンポーネント | `packages/vue/src/Popover.vue` |
| ルート Vue コンポーネント | `packages/starters/vue/src/App.vue` |

## 要件

- macOS 10.15+
- Node.js 18+
- Rust (stable)
- Aerospace ウィンドウマネージャー（ワークスペース機能用、オプション）
