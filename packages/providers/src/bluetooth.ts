import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { BluetoothInfo, Provider } from './types'

export interface BluetoothProvider extends Provider<BluetoothInfo> {
  getBluetooth(): Promise<BluetoothInfo>
  onBluetoothChange(callback: (info: BluetoothInfo) => void): () => void
  toggle(): Promise<void>
}

export function createBluetoothProvider(): BluetoothProvider {
  let unlistenFn: UnlistenFn | null = null
  let subscribers: Set<(info: BluetoothInfo) => void> = new Set()

  const setupListener = async () => {
    if (unlistenFn) return

    unlistenFn = await listen<BluetoothInfo>('bluetooth-changed', (event) => {
      subscribers.forEach((callback) => callback(event.payload))
    })
  }

  return {
    async get() {
      return this.getBluetooth()
    },

    async getBluetooth() {
      return invoke<BluetoothInfo>('get_bluetooth_info')
    },

    subscribe(callback) {
      return this.onBluetoothChange(callback)
    },

    onBluetoothChange(callback) {
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

    async toggle() {
      return invoke('toggle_bluetooth')
    }
  }
}
