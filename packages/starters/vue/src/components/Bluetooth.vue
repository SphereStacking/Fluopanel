<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { Icon } from '@iconify/vue'
import type { BluetoothProvider, BluetoothInfo } from '@arcana/providers'

const props = defineProps<{
  provider: BluetoothProvider
}>()

const bluetooth = ref<BluetoothInfo | null>(null)
let unsubscribe: (() => void) | null = null

const refresh = async () => {
  try {
    bluetooth.value = await props.provider.getBluetooth()
  } catch (error) {
    console.error('Failed to get bluetooth info:', error)
  }
}

onMounted(async () => {
  await refresh()
  unsubscribe = props.provider.onBluetoothChange((info) => {
    bluetooth.value = info
  })
})

onUnmounted(() => {
  unsubscribe?.()
})

const icon = computed(() => {
  if (!bluetooth.value) return 'mdi:bluetooth'
  if (!bluetooth.value.enabled) return 'mdi:bluetooth-off'
  const connectedDevices = bluetooth.value.devices.filter(d => d.connected)
  if (connectedDevices.length > 0) return 'mdi:bluetooth-connect'
  return 'mdi:bluetooth'
})

const connectedCount = computed(() => {
  if (!bluetooth.value || !bluetooth.value.enabled) return 0
  return bluetooth.value.devices.filter(d => d.connected).length
})

const statusColor = computed(() => {
  if (!bluetooth.value || !bluetooth.value.enabled) return 'text-[var(--text-ghost)]'
  if (connectedCount.value > 0) return 'text-[var(--holo-blue)]'
  return 'text-[var(--text-secondary)]'
})

const toggle = async () => {
  try {
    await props.provider.toggle()
    await refresh()
  } catch (error) {
    console.error('Failed to toggle bluetooth:', error)
  }
}
</script>

<template>
  <div
    v-if="bluetooth"
    class="
      flex items-center gap-1.5 py-1 px-2.5 rounded-lg
      text-[12px] tracking-wide
      transition-all duration-200
      hover:bg-[var(--widget-glass-hover)]
      cursor-pointer
      group
    "
    @click="toggle"
  >
    <Icon
      :icon="icon"
      class="w-[14px] h-[14px] transition-colors duration-200"
      :class="statusColor"
    />
    <span
      v-if="connectedCount > 0"
      class="font-medium tabular-nums text-[var(--text-secondary)] group-hover:text-[var(--text-primary)] transition-colors duration-200"
    >{{ connectedCount }}</span>
  </div>
</template>
