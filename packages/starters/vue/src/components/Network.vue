<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { Icon } from '@iconify/vue'
import type { NetworkProvider, NetworkInfo } from '@arcana/providers'

const props = defineProps<{
  provider: NetworkProvider
}>()

const network = ref<NetworkInfo | null>(null)
let intervalId: ReturnType<typeof setInterval> | null = null

const refresh = async () => {
  try {
    network.value = await props.provider.getNetwork()
  } catch (error) {
    console.error('Failed to get network info:', error)
  }
}

onMounted(() => {
  refresh()
  intervalId = setInterval(refresh, 10000)
})

onUnmounted(() => {
  if (intervalId) clearInterval(intervalId)
})

const icon = computed(() => {
  if (!network.value || !network.value.connected) return 'mdi:wifi-off'
  if (network.value.type === 'wifi') {
    const strength = network.value.signalStrength ?? 100
    if (strength > 75) return 'mdi:wifi-strength-4'
    if (strength > 50) return 'mdi:wifi-strength-3'
    if (strength > 25) return 'mdi:wifi-strength-2'
    return 'mdi:wifi-strength-1'
  }
  return 'mdi:ethernet'
})

const displayText = computed(() => {
  if (!network.value || !network.value.connected) return 'Offline'
  if (network.value.type === 'wifi' && network.value.ssid) {
    return network.value.ssid
  }
  return network.value.type === 'wifi' ? 'WiFi' : 'Ethernet'
})

const statusColor = computed(() => {
  if (!network.value || !network.value.connected) return 'text-[var(--text-ghost)]'
  return 'text-[var(--holo-green)]'
})
</script>

<template>
  <div
    v-if="network"
    class="
      flex items-center gap-1.5 py-1 px-2.5 rounded-lg
      text-[12px] tracking-wide
      transition-all duration-200
      hover:bg-[var(--widget-glass-hover)]
      cursor-default
      group
    "
    :class="{ 'opacity-50': !network.connected }"
  >
    <Icon
      :icon="icon"
      class="w-[14px] h-[14px] transition-colors duration-200"
      :class="statusColor"
    />
    <span
      class="
        font-medium max-w-[80px] overflow-hidden text-ellipsis whitespace-nowrap
        text-[var(--text-secondary)] group-hover:text-[var(--text-primary)]
        transition-colors duration-200
      "
    >{{ displayText }}</span>
  </div>
</template>
