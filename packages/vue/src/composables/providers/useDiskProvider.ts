import { ref, onMounted, onUnmounted } from 'vue'
import { createDiskProvider, type DiskInfo, type DiskProvider } from '@arcana/providers'

export function useDiskProvider(mountPoint = '/') {
  const data = ref<DiskInfo | null>(null)
  let provider: DiskProvider | null = null
  let intervalId: ReturnType<typeof setInterval> | null = null

  const refresh = async () => {
    try {
      data.value = await provider?.getDisk(mountPoint) ?? null
    } catch (error) {
      console.error('Failed to get disk info:', error)
    }
  }

  onMounted(() => {
    provider = createDiskProvider()
    refresh()
    intervalId = setInterval(refresh, 60000)
  })

  onUnmounted(() => {
    if (intervalId) clearInterval(intervalId)
  })

  return { data, refresh }
}
