import { ref, onMounted, onUnmounted } from 'vue'
import { createNetworkProvider, type NetworkInfo } from 'fluopanel-providers'

export function useNetworkProvider() {
  const data = ref<NetworkInfo | null>(null)
  let unsubscribe: (() => void) | null = null

  onMounted(async () => {
    const provider = createNetworkProvider()
    try {
      data.value = await provider.getNetwork()
    } catch (error) {
      console.error('Failed to get network info:', error)
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
