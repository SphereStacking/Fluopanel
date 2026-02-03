import { ref, onMounted, onUnmounted } from 'vue'
import { createVolumeProvider, type VolumeInfo, type VolumeProvider } from 'fluopanel-providers'

export function useVolumeProvider() {
  const data = ref<VolumeInfo | null>(null)
  let provider: VolumeProvider | null = null
  let unsubscribe: (() => void) | null = null

  onMounted(async () => {
    provider = createVolumeProvider()
    try {
      data.value = await provider.getVolume()
    } catch (error) {
      console.error('Failed to get volume info:', error)
    }
    unsubscribe = provider.subscribe((info) => {
      data.value = info
    })
  })

  onUnmounted(() => {
    unsubscribe?.()
  })

  const setVolume = async (level: number) => {
    await provider?.setVolume(level)
  }

  const mute = async () => {
    await provider?.mute()
  }

  const unmute = async () => {
    await provider?.unmute()
  }

  const toggleMute = async () => {
    await provider?.toggleMute()
  }

  return { data, setVolume, mute, unmute, toggleMute }
}
