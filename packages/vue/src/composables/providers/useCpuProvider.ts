import { ref, onMounted, onUnmounted } from 'vue'
import { createCpuProvider, type CpuInfo } from 'fluopanel-providers'

export function useCpuProvider() {
  const data = ref<CpuInfo | null>(null)
  let unsubscribe: (() => void) | null = null

  onMounted(async () => {
    const provider = createCpuProvider()
    try {
      data.value = await provider.getCpu()
    } catch (error) {
      console.error('Failed to get CPU info:', error)
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
