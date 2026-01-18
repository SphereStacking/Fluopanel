import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

/**
 * SharedStore interface for cross-window state sharing
 */
export interface SharedStore<T> {
  /** Get current value from store */
  get(): Promise<T | null>
  /** Set value and broadcast to all windows */
  set(value: T): Promise<void>
  /** Subscribe to value changes from other windows */
  subscribe(callback: (value: T) => void): () => void
  /** Delete value from store */
  delete(): Promise<void>
}

/**
 * Create a shared store instance for cross-window state sharing.
 *
 * This provides a framework-agnostic API for sharing state between
 * Tauri windows. The state is stored in Rust and changes are
 * automatically broadcast to all windows via Tauri events.
 *
 * @param key - Unique identifier for this store
 * @returns SharedStore instance
 *
 * @example
 * ```ts
 * const githubStore = createSharedStore<GitHubState>('github')
 *
 * // Set value (broadcasts to all windows)
 * await githubStore.set({ issues: [...], notifications: [...] })
 *
 * // Get current value
 * const data = await githubStore.get()
 *
 * // Subscribe to changes from other windows
 * const unsubscribe = githubStore.subscribe((newData) => {
 *   console.log('Data updated:', newData)
 * })
 * ```
 */
export function createSharedStore<T>(key: string): SharedStore<T> {
  let unlistenFn: UnlistenFn | null = null
  const subscribers = new Set<(value: T) => void>()

  const setupListener = async () => {
    if (unlistenFn) return

    unlistenFn = await listen<T>(`store-changed:${key}`, (event) => {
      subscribers.forEach((cb) => cb(event.payload))
    })
  }

  return {
    async get(): Promise<T | null> {
      const result = await invoke<T | null>('store_get', { key })
      return result
    },

    async set(value: T): Promise<void> {
      await invoke('store_set', { key, value })
    },

    subscribe(callback: (value: T) => void): () => void {
      subscribers.add(callback)

      // Start listener on first subscriber
      if (subscribers.size === 1) {
        setupListener()
      }

      // Return unsubscribe function
      return () => {
        subscribers.delete(callback)

        // Stop listener when no subscribers remain
        if (subscribers.size === 0 && unlistenFn) {
          unlistenFn()
          unlistenFn = null
        }
      }
    },

    async delete(): Promise<void> {
      await invoke('store_delete', { key })
    },
  }
}

/**
 * Get all keys currently in the shared store
 */
export async function getSharedStoreKeys(): Promise<string[]> {
  return await invoke<string[]>('store_keys')
}
