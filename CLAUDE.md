# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

Arcana は Tauri 2 (Rust バックエンド) で構築された macOS 用カスタマイズ可能なウィジェットフレームワークです。ユーザーは Vue/React/HTML でカスタムウィジェットを作成し、メニューバー型やフローティング型のウィジェットを画面に配置できます。

## 設計原則

### 後方互換性より Clean Architecture
- **後方互換性は不要。よりクリーンな構成を常に優先する**
- レガシーコードや古い設計パターンを維持する必要はない
- 破壊的変更を恐れず、最適な設計を追求する
- 「動いているから触らない」ではなく「より良くできるなら変える」

### フレームワーク非依存
- **Core パッケージ (`@arcana/core`) はフレームワーク非依存であること**
- Vue, React, Solid など複数のフレームワークで動作可能にする
- フレームワーク固有のコード（ライフサイクル、リアクティビティ）は各 starter に配置
- Core には純粋な TypeScript ロジックのみ

### アーキテクチャ層
```
┌─────────────────────────────────────┐
│  User Widgets (Vue/React/HTML)     │  ← ユーザー定義ウィジェット
├─────────────────────────────────────┤
│  Starters (Vue/React/Solid)         │  ← フレームワーク固有ラッパー
├─────────────────────────────────────┤
│  Providers (TypeScript)             │  ← システムデータプロバイダー
├─────────────────────────────────────┤
│  Core (TypeScript)                  │  ← WidgetManager, WindowController
├─────────────────────────────────────┤
│  Tauri (Rust)                       │  ← Widget Discovery, 動的ウィンドウ
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
├── core/              # ウィジェット管理 (WidgetManager, WindowController)
├── providers/         # システムデータプロバイダー (10種類)
├── vue/               # Vue 用ユーティリティ
├── tauri/             # Tauri デスクトップアプリ (Rust)
│   └── src-tauri/src/
│       ├── commands/  # Tauri IPC コマンド
│       ├── ipc/       # Unix ソケット IPC
│       └── lib.rs     # エントリーポイント
└── starters/
    └── vue/           # Vue 3 フロントエンド実装
```

### ユーザーウィジェット構造

```
~/.config/arcana/
├── config.json           # グローバル設定
└── widgets/              # ユーザー定義ウィジェット
    └── my-widget/
        ├── widget.json   # ウィジェットマニフェスト
        ├── index.html    # エントリーポイント
        └── dist/         # ビルド済みアセット
```

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
| サードパーティウィジェット | `arcana-widget-{name}` |

### IPC レイヤー

フロントエンドは Tauri コマンドを通じて Rust と通信します。Rust コマンドは `packages/tauri/src-tauri/src/commands/` にあります:
- `aerospace.rs` - ワークスペース管理 (Aerospace CLI をラップ)
- `system.rs` - バッテリー、CPU、メモリ、ネットワーク、ボリューム、メディア、ディスク情報
- `icons.rs` - アプリアイコン抽出 (base64 エンコード)
- `config.rs` - 設定の永続化
- `window.rs` - ウィンドウ配置/ジオメトリ

### CLI & IPC

Unix ソケット (`/tmp/arcana.sock`) を使用した IPC:

```bash
arcana emit workspace-changed   # ワークスペース変更通知
```

- Aerospace の `exec-on-workspace-change` から呼び出し可能
- 既存インスタンスにイベント送信

### カスタムプロトコル

アプリは `arcana://` プロトコルを使用して以下を配信:
1. `~/.config/arcana/dist/` からのユーザー設定（存在する場合）
2. フォールバックとしてバンドルされたアセット

### ウィンドウ動作

- フレームレス、透明、常に最前面
- Dock から非表示（`cocoa` クレートによるアクセサリアプリ）
- モニター変更時に再配置（NSNotificationCenter オブザーバー）
- CSS absolute 風の position 指定（top, left, right, bottom, width, height）
- px, vh, vw, calc() 対応

## 主要ファイル

| 用途 | 場所 |
|------|------|
| ウィジェット型定義 | `packages/core/src/types.ts` |
| プロバイダー型定義 | `packages/providers/src/types.ts` |
| Tauri セットアップ & イベント | `packages/tauri/src-tauri/src/lib.rs` |
| IPC コマンド | `packages/tauri/src-tauri/src/commands/*.rs` |
| IPC サーバー | `packages/tauri/src-tauri/src/ipc/mod.rs` |
| Tauri 設定 | `packages/tauri/src-tauri/tauri.conf.json` |
| ウィンドウ初期化 (Vue) | `packages/starters/vue/src/composables/useArcanaInit.ts` |
| ルート Vue コンポーネント | `packages/starters/vue/src/App.vue` |

## 要件

- macOS 10.15+
- Node.js 18+
- Rust (stable)
- Aerospace ウィンドウマネージャー（ワークスペース機能用）
