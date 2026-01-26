# HTML Widget Template

HTML + JavaScript (ESM) を使ったシンプルなウィジェットテンプレート。
ビルド不要で、そのまま動作します。

## 使い方

1. このフォルダを `~/.config/arcana/widgets/` にコピー
2. `widget.json` の `id` と `name` を変更
3. `index.html` を編集

## ファイル構成

```
my-widget/
├── widget.json   # ウィジェット設定
├── index.html    # メインファイル
└── style.css     # スタイル（オプション）
```

## importmap

Arcana が自動的に importmap を注入するため、以下のインポートが使えます：

```javascript
import { ... } from '@arcana/providers'
import { ... } from 'vue'
import { invoke } from '@tauri-apps/api/core'
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

## プロバイダーの使い方

```javascript
const provider = createBatteryProvider()

// 一度だけ取得
const data = await provider.get()

// 更新を購読
const unsubscribe = provider.subscribe((data) => {
  console.log(data)
})

// 購読解除
unsubscribe()
```
