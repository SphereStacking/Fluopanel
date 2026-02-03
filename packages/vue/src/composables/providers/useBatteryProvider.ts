import { ref, onMounted, onUnmounted } from 'vue'
import { createBatteryProvider, type BatteryInfo } from 'fluopanel-providers'

export function useBatteryProvider() {
  const data = ref<BatteryInfo | null>(null)
  let unsubscribe: (() => void) | null = null

  onMounted(async () => {
    const provider = createBatteryProvider()
    try {
      data.value = await provider.getBattery()
    } catch (error) {
      console.error('Failed to get battery info:', error)
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
