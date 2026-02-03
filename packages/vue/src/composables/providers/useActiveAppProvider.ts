import { ref, onMounted, onUnmounted } from 'vue'
import { createActiveAppProvider, type ActiveAppInfo } from 'fluopanel-providers'

export function useActiveAppProvider() {
  const data = ref<ActiveAppInfo | null>(null)
  let unsubscribe: (() => void) | null = null

  onMounted(async () => {
    const provider = createActiveAppProvider()
    try {
      data.value = await provider.getActiveApp()
    } catch (error) {
      console.error('Failed to get active app info:', error)
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
