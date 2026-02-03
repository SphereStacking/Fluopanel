import { ref, onMounted, onUnmounted, type Ref } from 'vue'
import type { Provider } from 'fluopanel-providers'

export function useProvider<T>(provider: Provider<T>, initialValue: T): Ref<T> {
  const data = ref<T>(initialValue) as Ref<T>
  let unsubscribe: (() => void) | null = null

  onMounted(async () => {
    // Get initial data
    try {
      data.value = await provider.get()
    } catch (error) {
      console.error('Failed to get initial data:', error)
    }

    // Subscribe to updates
    unsubscribe = provider.subscribe((newData) => {
      data.value = newData
    })
  })

  onUnmounted(() => {
    if (unsubscribe) {
      unsubscribe()
    }
  })

  return data
}

export function usePollingProvider<T>(
  fetcher: () => Promise<T>,
  initialValue: T,
  interval: number = 1000
): Ref<T> {
  const data = ref<T>(initialValue) as Ref<T>
  let intervalId: ReturnType<typeof setInterval> | null = null

  const refresh = async () => {
    try {
      data.value = await fetcher()
    } catch (error) {
      console.error('Failed to fetch data:', error)
    }
  }

  onMounted(() => {
    refresh()
    intervalId = setInterval(refresh, interval)
  })

  onUnmounted(() => {
    if (intervalId) {
      clearInterval(intervalId)
    }
  })

  return data
}
