import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { BatteryInfo, Provider } from './types'

export interface BatteryProvider extends Provider<BatteryInfo> {
  getBattery(): Promise<BatteryInfo>
  onBatteryChange(callback: (info: BatteryInfo) => void): () => void
}

export function createBatteryProvider(): BatteryProvider {
  let unlistenFn: UnlistenFn | null = null
  let subscribers: Set<(info: BatteryInfo) => void> = new Set()

  const setupListener = async () => {
    if (unlistenFn) return

    unlistenFn = await listen<BatteryInfo>('battery-changed', (event) => {
      subscribers.forEach((callback) => callback(event.payload))
    })
  }

  return {
    async get() {
      return this.getBattery()
    },

    async getBattery() {
      return invoke<BatteryInfo>('get_battery_info')
    },

    subscribe(callback) {
      return this.onBatteryChange(callback)
    },

    onBatteryChange(callback) {
      subscribers.add(callback)
      setupListener()

      return () => {
        subscribers.delete(callback)
        if (subscribers.size === 0 && unlistenFn) {
          unlistenFn()
          unlistenFn = null
        }
      }
    }
  }
}
