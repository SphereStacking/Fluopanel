import { ref, onMounted, onUnmounted } from 'vue'
import { createYouTubeMusicProvider, type YouTubeMusicInfo } from 'fluopanel-providers'

export function useYouTubeMusicProvider() {
  const data = ref<YouTubeMusicInfo | null>(null)
  const provider = createYouTubeMusicProvider()
  let unsubscribe: (() => void) | null = null

  onMounted(async () => {
    try {
      data.value = await provider.getInfo()
    } catch (error) {
      console.error('Failed to get YouTube Music info:', error)
    }
    unsubscribe = provider.subscribe((info) => {
      data.value = info
    })
  })

  onUnmounted(() => {
    unsubscribe?.()
  })

  const toggle = async () => {
    await provider.toggle()
  }

  const next = async () => {
    await provider.next()
  }

  const previous = async () => {
    await provider.previous()
  }

  const seek = async (seconds: number) => {
    await provider.seek(seconds)
  }

  const launch = async () => {
    await provider.launch()
  }

  return { data, toggle, next, previous, seek, launch }
}
