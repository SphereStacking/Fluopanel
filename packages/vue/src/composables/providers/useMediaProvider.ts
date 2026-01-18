import { ref, onMounted, onUnmounted } from 'vue'
import { createMediaProvider, type MediaInfo, type MediaProvider } from '@arcana/providers'

export function useMediaProvider() {
  const data = ref<MediaInfo | null>(null)
  let provider: MediaProvider | null = null
  let unsubscribe: (() => void) | null = null

  onMounted(async () => {
    provider = createMediaProvider()
    try {
      data.value = await provider.getMedia()
    } catch (error) {
      console.error('Failed to get media info:', error)
    }
    unsubscribe = provider.subscribe((info) => {
      data.value = info
    })
  })

  onUnmounted(() => {
    unsubscribe?.()
  })

  const play = async () => {
    await provider?.play()
  }

  const pause = async () => {
    await provider?.pause()
  }

  const next = async () => {
    await provider?.next()
  }

  const previous = async () => {
    await provider?.previous()
  }

  return { data, play, pause, next, previous }
}
