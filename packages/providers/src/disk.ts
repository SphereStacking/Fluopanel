import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { DiskInfo, Provider } from './types'

export interface DiskProvider extends Provider<DiskInfo[]> {
  getDisks(): Promise<DiskInfo[]>
  getDisk(mountPoint?: string): Promise<DiskInfo>
  onDiskChange(callback: (info: DiskInfo[]) => void): () => void
}

export function createDiskProvider(): DiskProvider {
  let unlistenFn: UnlistenFn | null = null
  let subscribers: Set<(info: DiskInfo[]) => void> = new Set()

  const setupListener = async () => {
    if (unlistenFn) return

    unlistenFn = await listen<DiskInfo[]>('disk-changed', (event) => {
      subscribers.forEach((callback) => callback(event.payload))
    })
  }

  return {
    async get() {
      return this.getDisks()
    },

    async getDisks() {
      return invoke<DiskInfo[]>('get_disk_info')
    },

    async getDisk(mountPoint = '/') {
      const disks = await this.getDisks()
      return disks.find(d => d.mountPoint === mountPoint) ?? disks[0]
    },

    subscribe(callback) {
      return this.onDiskChange(callback)
    },

    onDiskChange(callback) {
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
