import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { VolumeInfo, Provider } from './types'

export interface VolumeProvider extends Provider<VolumeInfo> {
  getVolume(): Promise<VolumeInfo>
  onVolumeChange(callback: (info: VolumeInfo) => void): () => void
  setVolume(level: number): Promise<void>
  mute(): Promise<void>
  unmute(): Promise<void>
  toggleMute(): Promise<void>
}

export function createVolumeProvider(): VolumeProvider {
  let unlistenFn: UnlistenFn | null = null
  let subscribers: Set<(info: VolumeInfo) => void> = new Set()

  const setupListener = async () => {
    if (unlistenFn) return

    unlistenFn = await listen<VolumeInfo>('volume-changed', (event) => {
      subscribers.forEach((callback) => callback(event.payload))
    })
  }

  return {
    async get() {
      return this.getVolume()
    },

    async getVolume() {
      return invoke<VolumeInfo>('get_volume_info')
    },

    subscribe(callback) {
      return this.onVolumeChange(callback)
    },

    onVolumeChange(callback) {
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

    async setVolume(level: number) {
      return invoke('set_volume', { level: Math.max(0, Math.min(100, level)) })
    },

    async mute() {
      return invoke('set_mute', { muted: true })
    },

    async unmute() {
      return invoke('set_mute', { muted: false })
    },

    async toggleMute() {
      return invoke('toggle_mute')
    }
  }
}
