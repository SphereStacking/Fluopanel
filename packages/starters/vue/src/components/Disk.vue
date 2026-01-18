<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { Icon } from '@iconify/vue'
import type { DiskProvider, DiskInfo } from '@arcana/providers'

const props = defineProps<{
  provider: DiskProvider
}>()

const disk = ref<DiskInfo | null>(null)
let intervalId: ReturnType<typeof setInterval> | null = null

const refresh = async () => {
  try {
    disk.value = await props.provider.getDisk('/')
  } catch (error) {
    console.error('Failed to get disk info:', error)
  }
}

onMounted(() => {
  refresh()
  intervalId = setInterval(refresh, 60000)
})

onUnmounted(() => {
  if (intervalId) clearInterval(intervalId)
})

const availableText = computed(() => {
  if (!disk.value) return ''
  const gb = disk.value.available / (1024 * 1024 * 1024)
  return `${gb.toFixed(0)}G`
})

const statusColor = computed(() => {
  if (!disk.value) return 'text-[var(--text-secondary)]'
  if (disk.value.usage > 90) return 'text-[var(--danger)]'
  if (disk.value.usage > 75) return 'text-[var(--warning)]'
  return 'text-[var(--holo-orange)]'
})

const glowClass = computed(() => {
  if (!disk.value) return ''
  if (disk.value.usage > 90) return 'shadow-[0_0_8px_rgba(248,113,113,0.4)]'
  return ''
})
</script>

<template>
  <div
    v-if="disk"
    class="
      flex items-center gap-1.5 py-1 px-2.5 rounded-lg
      text-[12px] tracking-wide
      transition-all duration-200
      hover:bg-[var(--widget-glass-hover)]
      cursor-default
      group
    "
    :class="glowClass"
  >
    <Icon icon="mdi:harddisk" class="w-[14px] h-[14px] text-[var(--holo-orange)] transition-colors duration-200" />
    <span
      class="font-medium tabular-nums min-w-[3.5ch] text-right transition-colors duration-200"
      :class="statusColor"
    >{{ availableText }}</span>
  </div>
</template>
