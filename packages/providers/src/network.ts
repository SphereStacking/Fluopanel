import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { NetworkInfo, Provider } from './types'

export interface NetworkProvider extends Provider<NetworkInfo> {
  getNetwork(): Promise<NetworkInfo>
  onNetworkChange(callback: (info: NetworkInfo) => void): () => void
}

export function createNetworkProvider(): NetworkProvider {
  let unlistenFn: UnlistenFn | null = null
  let subscribers: Set<(info: NetworkInfo) => void> = new Set()

  const setupListener = async () => {
    if (unlistenFn) return

    unlistenFn = await listen<NetworkInfo>('network-changed', (event) => {
      subscribers.forEach((callback) => callback(event.payload))
    })
  }

  return {
    async get() {
      return this.getNetwork()
    },

    async getNetwork() {
      return invoke<NetworkInfo>('get_network_info')
    },

    subscribe(callback) {
      return this.onNetworkChange(callback)
    },

    onNetworkChange(callback) {
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
