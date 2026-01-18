import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { MediaInfo, Provider } from './types'

export interface MediaProvider extends Provider<MediaInfo> {
  getMedia(): Promise<MediaInfo>
  onMediaChange(callback: (info: MediaInfo) => void): () => void
  play(): Promise<void>
  pause(): Promise<void>
  next(): Promise<void>
  previous(): Promise<void>
}

export function createMediaProvider(): MediaProvider {
  let unlistenFn: UnlistenFn | null = null
  let subscribers: Set<(info: MediaInfo) => void> = new Set()

  const setupListener = async () => {
    if (unlistenFn) return

    unlistenFn = await listen<MediaInfo>('media-changed', (event) => {
      subscribers.forEach((callback) => callback(event.payload))
    })
  }

  return {
    async get() {
      return this.getMedia()
    },

    async getMedia() {
      return invoke<MediaInfo>('get_media_info')
    },

    subscribe(callback) {
      return this.onMediaChange(callback)
    },

    onMediaChange(callback) {
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

    async play() {
      return invoke('media_play')
    },

    async pause() {
      return invoke('media_pause')
    },

    async next() {
      return invoke('media_next')
    },

    async previous() {
      return invoke('media_previous')
    }
  }
}
