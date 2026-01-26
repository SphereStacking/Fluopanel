# Vue Widget Template

Vue SFC (Single File Component) を使ったウィジェットテンプレート。

## 使い方

1. このフォルダを `~/.config/arcana/widgets/` にコピー
2. `widget.json` の `id` と `name` を変更
3. `App.vue` を編集

## ファイル構成

```
my-widget/
├── widget.json   # ウィジェット設定
├── App.vue       # メインコンポーネント
└── .arcana/      # ビルド出力（自動生成）
```

## 利用可能なプロバイダー

```javascript
import {
  createBatteryProvider,
  createCpuProvider,
  createMemoryProvider,
  createNetworkProvider,
  createDiskProvider,
  createVolumeProvider,
  createMediaProvider,
  createActiveAppProvider,
  createDateProvider,
  createAerospaceProvider,
} from '@arcana/providers'
```

## 自動ビルド

`.vue` ファイルを保存すると自動的にビルドされます。
