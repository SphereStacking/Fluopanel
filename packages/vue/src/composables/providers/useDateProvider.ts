import { ref, onMounted, onUnmounted } from 'vue'
import { createDateProvider, type DateInfo } from '@arcana/providers'

export function useDateProvider(format = 'HH:mm', interval = 60000) {
  const data = ref<DateInfo>({ timestamp: 0, formatted: '' })
  let unsubscribe: (() => void) | null = null

  onMounted(() => {
    const provider = createDateProvider()
    data.value = provider.getDate(format)
    unsubscribe = provider.startPolling((info) => {
      data.value = info
    }, interval)
  })

  onUnmounted(() => {
    unsubscribe?.()
  })

  return { data }
}
