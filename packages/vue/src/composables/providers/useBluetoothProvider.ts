import { ref, onMounted, onUnmounted } from 'vue'
import { createBluetoothProvider, type BluetoothInfo, type BluetoothProvider } from 'fluopanel-providers'

export function useBluetoothProvider() {
  const data = ref<BluetoothInfo | null>(null)
  let provider: BluetoothProvider | null = null
  let unsubscribe: (() => void) | null = null

  const refresh = async () => {
    try {
      data.value = await provider?.getBluetooth() ?? null
    } catch (error) {
      console.error('Failed to get bluetooth info:', error)
    }
  }

  onMounted(async () => {
    provider = createBluetoothProvider()
    await refresh()
    unsubscribe = provider.subscribe((info) => {
      data.value = info
    })
  })

  onUnmounted(() => {
    unsubscribe?.()
  })

  const toggle = async () => {
    await provider?.toggle()
    await refresh()
  }

  return { data, toggle, refresh }
}
