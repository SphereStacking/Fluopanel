import { ref, onMounted, onUnmounted } from 'vue'
import { createBrightnessProvider, type BrightnessInfo, type BrightnessProvider } from '@arcana/providers'

export function useBrightnessProvider() {
  const data = ref<BrightnessInfo | null>(null)
  let provider: BrightnessProvider | null = null
  let unsubscribe: (() => void) | null = null

  onMounted(async () => {
    provider = createBrightnessProvider()
    try {
      data.value = await provider.getBrightness()
    } catch (error) {
      console.error('Failed to get brightness info:', error)
    }
    unsubscribe = provider.subscribe((info) => {
      data.value = info
    })
  })

  onUnmounted(() => {
    unsubscribe?.()
  })

  const setBrightness = async (level: number) => {
    await provider?.setBrightness(level)
  }

  return { data, setBrightness }
}
