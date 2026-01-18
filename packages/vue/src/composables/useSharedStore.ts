import { ref, onUnmounted, type Ref } from 'vue'
import { createSharedStore, type SharedStore } from '@arcana/core'

/**
 * Return type for useSharedStore composable
 */
export interface UseSharedStoreReturn<T> {
  /** Reactive data from the store (null if not loaded or empty) */
  data: Ref<T | null>
  /** Loading state for initial fetch */
  loading: Ref<boolean>
  /** Set value and broadcast to all windows */
  set: (value: T) => Promise<void>
  /** Refresh data from store */
  refresh: () => Promise<void>
  /** Delete value from store */
  delete: () => Promise<void>
  /** The underlying store instance */
  store: SharedStore<T>
}

/**
 * Vue composable for cross-window shared state.
 *
 * Provides reactive state that automatically syncs across all Tauri windows.
 * When any window calls `set()`, all other windows receive the update
 * automatically via their `data` ref.
 *
 * @param key - Unique identifier for this store
 * @returns Reactive store interface
 *
 * @example
 * ```vue
 * <script setup lang="ts">
 * import { useSharedStore } from '@arcana/vue'
 *
 * interface GitHubState {
 *   issues: Issue[]
 *   notifications: Notification[]
 * }
 *
 * const { data, set, loading } = useSharedStore<GitHubState>('github')
 *
 * // data.value is reactive and auto-updates from other windows
 * // set() broadcasts to all windows
 * async function refresh() {
 *   const freshData = await fetchFromGitHub()
 *   await set(freshData)
 * }
 * </script>
 *
 * <template>
 *   <div v-if="loading">Loading...</div>
 *   <div v-else-if="data">
 *     {{ data.issues.length }} issues
 *   </div>
 * </template>
 * ```
 */
export function useSharedStore<T>(key: string): UseSharedStoreReturn<T> {
  const store = createSharedStore<T>(key)
  const data = ref<T | null>(null) as Ref<T | null>
  const loading = ref(true)

  // Fetch initial value
  store.get().then((value) => {
    data.value = value
    loading.value = false
  })

  // Subscribe to updates from other windows
  const unsubscribe = store.subscribe((value) => {
    data.value = value
  })

  // Cleanup on component unmount
  onUnmounted(() => {
    unsubscribe()
  })

  return {
    data,
    loading,

    async set(value: T): Promise<void> {
      await store.set(value)
      // Also update local state immediately
      data.value = value
    },

    async refresh(): Promise<void> {
      loading.value = true
      data.value = await store.get()
      loading.value = false
    },

    async delete(): Promise<void> {
      await store.delete()
      data.value = null
    },

    store,
  }
}
