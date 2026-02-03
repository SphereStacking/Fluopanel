import { ref, onMounted, onUnmounted } from 'vue'
import { createMemoryProvider, type MemoryInfo } from 'fluopanel-providers'

export function useMemoryProvider() {
  const data = ref<MemoryInfo | null>(null)
  let unsubscribe: (() => void) | null = null

  onMounted(async () => {
    const provider = createMemoryProvider()
    try {
      data.value = await provider.getMemory()
    } catch (error) {
      console.error('Failed to get memory info:', error)
    }
    unsubscribe = provider.subscribe((info) => {
      data.value = info
    })
  })

  onUnmounted(() => {
    unsubscribe?.()
  })

  return { data }
}
