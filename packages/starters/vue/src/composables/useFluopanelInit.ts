import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { FluopanelInitOptions, CSSValue } from '../types/fluopanel'
import type { MonitorInfo } from 'fluopanel-core'

/**
 * calc() 式を評価する
 * 対応: calc(100vh - 32px), calc(100vw - 48px) など
 */
function evaluateCalc(expr: string, monitor: MonitorInfo): number {
  // calc(...) から中身を取り出す
  const inner = expr.slice(5, -1).trim()

  // 簡易パーサー: "100vh - 32px" のような形式を処理
  const tokens = inner.split(/\s*([-+])\s*/)

  let result = 0
  let operator = '+'

  for (const token of tokens) {
    if (token === '+' || token === '-') {
      operator = token
      continue
    }

    let value = 0
    const trimmed = token.trim()

    if (trimmed.endsWith('vh')) {
      value = (parseFloat(trimmed) / 100) * monitor.height
    } else if (trimmed.endsWith('vw')) {
      value = (parseFloat(trimmed) / 100) * monitor.width
    } else if (trimmed.endsWith('px')) {
      value = parseFloat(trimmed)
    } else if (!isNaN(Number(trimmed))) {
      value = Number(trimmed)
    }

    if (operator === '+') {
      result += value
    } else {
      result -= value
    }
  }

  return result
}

/**
 * CSS値をピクセルに変換
 */
function toPixels(
  value: CSSValue | undefined,
  _dimension: 'width' | 'height',
  monitor: MonitorInfo
): number | undefined {
  if (value === undefined) return undefined
  if (typeof value === 'number') return value

  const strValue = String(value)
  if (strValue.endsWith('px')) return parseFloat(strValue)
  if (strValue.endsWith('vw')) return (parseFloat(strValue) / 100) * monitor.width
  if (strValue.endsWith('vh')) return (parseFloat(strValue) / 100) * monitor.height
  if (strValue.startsWith('calc(')) return evaluateCalc(strValue, monitor)

  return 0
}

/**
 * Fluopanel 初期化 composable
 * ウィンドウ位置・サイズをCSS absoluteライクに指定可能
 */
export function useFluopanelInit(options: FluopanelInitOptions) {
  const monitors = ref<MonitorInfo[]>([])
  let unlisten: UnlistenFn | null = null

  // モニター情報取得
  async function fetchMonitors(): Promise<void> {
    monitors.value = await invoke<MonitorInfo[]>('get_monitors')
  }

  // モニター名から検索（見つからなければ最初のモニター）
  function findMonitor(name?: string): MonitorInfo {
    if (!name) return monitors.value[0]
    return monitors.value.find((m) => m.name === name) ?? monitors.value[0]
  }

  // ウィンドウ配置を適用
  async function applyWindowGeometry(): Promise<void> {
    await fetchMonitors()

    if (monitors.value.length === 0) {
      console.warn('No monitors found')
      return
    }

    const pos = options.position
    const monitor = findMonitor(pos.monitor)

    // CSS値をピクセルに変換
    const top = toPixels(pos.top, 'height', monitor)
    const left = toPixels(pos.left, 'width', monitor)
    const right = toPixels(pos.right, 'width', monitor)
    const bottom = toPixels(pos.bottom, 'height', monitor)
    let width = toPixels(pos.width, 'width', monitor)
    let height = toPixels(pos.height, 'height', monitor)

    // CSS absoluteと同じ計算ロジック
    // left + right → width自動計算
    if (left !== undefined && right !== undefined && width === undefined) {
      width = monitor.width - left - right
    }
    // top + bottom → height自動計算
    if (top !== undefined && bottom !== undefined && height === undefined) {
      height = monitor.height - top - bottom
    }

    // x, y座標を計算
    let x: number
    let y: number

    if (left !== undefined) {
      x = left
    } else if (right !== undefined && width !== undefined) {
      x = monitor.width - right - width
    } else {
      x = 0
    }

    if (top !== undefined) {
      y = top
    } else if (bottom !== undefined && height !== undefined) {
      y = monitor.height - bottom - height
    } else {
      y = 0
    }

    await invoke('set_window_geometry', {
      x: monitor.x + x,
      y: monitor.y + y,
      width: width ?? monitor.width,
      height: height ?? 32,
    })
  }

  // 初期化時に自動適用 + モニター変更イベント購読
  onMounted(async () => {
    await applyWindowGeometry()

    // Rust からのモニター変更イベントを購読
    unlisten = await listen('monitor-changed', async () => {
      await applyWindowGeometry()
    })
  })

  // クリーンアップ
  onUnmounted(() => {
    unlisten?.()
  })

  return { applyWindowGeometry, monitors }
}
