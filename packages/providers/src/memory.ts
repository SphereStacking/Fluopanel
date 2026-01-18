import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { MemoryInfo, Provider } from './types'

export interface MemoryProvider extends Provider<MemoryInfo> {
  getMemory(): Promise<MemoryInfo>
  onMemoryChange(callback: (info: MemoryInfo) => void): () => void
}

export function createMemoryProvider(): MemoryProvider {
  let unlistenFn: UnlistenFn | null = null
  let subscribers: Set<(info: MemoryInfo) => void> = new Set()

  const setupListener = async () => {
    if (unlistenFn) return

    unlistenFn = await listen<MemoryInfo>('memory-changed', (event) => {
      subscribers.forEach((callback) => callback(event.payload))
    })
  }

  return {
    async get() {
      return this.getMemory()
    },

    async getMemory() {
      return invoke<MemoryInfo>('get_memory_info')
    },

    subscribe(callback) {
      return this.onMemoryChange(callback)
    },

    onMemoryChange(callback) {
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
