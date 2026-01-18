import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { CpuInfo, Provider } from './types'

export interface CpuProvider extends Provider<CpuInfo> {
  getCpu(): Promise<CpuInfo>
  onCpuChange(callback: (info: CpuInfo) => void): () => void
}

export function createCpuProvider(): CpuProvider {
  let unlistenFn: UnlistenFn | null = null
  let subscribers: Set<(info: CpuInfo) => void> = new Set()

  const setupListener = async () => {
    if (unlistenFn) return

    unlistenFn = await listen<CpuInfo>('cpu-changed', (event) => {
      subscribers.forEach((callback) => callback(event.payload))
    })
  }

  return {
    async get() {
      return this.getCpu()
    },

    async getCpu() {
      return invoke<CpuInfo>('get_cpu_info')
    },

    subscribe(callback) {
      return this.onCpuChange(callback)
    },

    onCpuChange(callback) {
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
