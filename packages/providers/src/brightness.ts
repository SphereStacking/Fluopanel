import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { BrightnessInfo, Provider } from './types'

export interface BrightnessProvider extends Provider<BrightnessInfo> {
  getBrightness(): Promise<BrightnessInfo>
  onBrightnessChange(callback: (info: BrightnessInfo) => void): () => void
  setBrightness(level: number): Promise<void>
}

export function createBrightnessProvider(): BrightnessProvider {
  let unlistenFn: UnlistenFn | null = null
  let subscribers: Set<(info: BrightnessInfo) => void> = new Set()

  const setupListener = async () => {
    if (unlistenFn) return

    unlistenFn = await listen<BrightnessInfo>('brightness-changed', (event) => {
      subscribers.forEach((callback) => callback(event.payload))
    })
  }

  return {
    async get() {
      return this.getBrightness()
    },

    async getBrightness() {
      return invoke<BrightnessInfo>('get_brightness_info')
    },

    subscribe(callback) {
      return this.onBrightnessChange(callback)
    },

    onBrightnessChange(callback) {
      subscribers.add(callback)
      setupListener()

      return () => {
        subscribers.delete(callback)
        if (subscribers.size === 0 && unlistenFn) {
          unlistenFn()
          unlistenFn = null
        }
      }
    },

    async setBrightness(level: number) {
      // level is 0-100, convert to 0.0-1.0 for Rust
      return invoke('set_brightness', { brightness: Math.max(0, Math.min(100, level)) / 100 })
    }
  }
}
