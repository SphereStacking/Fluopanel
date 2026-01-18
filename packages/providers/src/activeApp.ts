import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { ActiveAppInfo, Provider } from './types'

export interface ActiveAppProvider extends Provider<ActiveAppInfo> {
  getActiveApp(): Promise<ActiveAppInfo>
  onActiveAppChange(callback: (info: ActiveAppInfo) => void): () => void
}

export function createActiveAppProvider(): ActiveAppProvider {
  let unlistenFn: UnlistenFn | null = null
  let subscribers: Set<(info: ActiveAppInfo) => void> = new Set()

  const setupListener = async () => {
    if (unlistenFn) return

    unlistenFn = await listen<ActiveAppInfo>('active-app-changed', (event) => {
      subscribers.forEach((callback) => callback(event.payload))
    })
  }

  return {
    async get() {
      return this.getActiveApp()
    },

    async getActiveApp() {
      return invoke<ActiveAppInfo>('get_active_app_info')
    },

    subscribe(callback) {
      return this.onActiveAppChange(callback)
    },

    onActiveAppChange(callback) {
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
