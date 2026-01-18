<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { Icon } from '@iconify/vue'
import type { MemoryProvider, MemoryInfo } from '@arcana/providers'

const props = defineProps<{
  provider: MemoryProvider
}>()

const memory = ref<MemoryInfo | null>(null)
let intervalId: ReturnType<typeof setInterval> | null = null

const refresh = async () => {
  try {
    memory.value = await props.provider.getMemory()
  } catch (error) {
    console.error('Failed to get memory info:', error)
  }
}

onMounted(() => {
  refresh()
  intervalId = setInterval(refresh, 5000)
})

onUnmounted(() => {
  if (intervalId) clearInterval(intervalId)
})

const formatBytes = (bytes: number) => {
  const gb = bytes / (1024 * 1024 * 1024)
  return `${gb.toFixed(1)}G`
}

const usageText = computed(() => {
  if (!memory.value) return '—'
  return formatBytes(memory.value.used)
})

const statusColor = computed(() => {
  if (!memory.value) return 'text-[var(--text-secondary)]'
  if (memory.value.usage > 80) return 'text-[var(--danger)]'
  if (memory.value.usage > 60) return 'text-[var(--warning)]'
  return 'text-[var(--holo-purple)]'
})

const glowClass = computed(() => {
  if (!memory.value) return ''
  if (memory.value.usage > 80) return 'shadow-[0_0_8px_rgba(248,113,113,0.4)]'
  return ''
})
</script>

<template>
  <div
    v-if="memory"
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
    <Icon icon="mdi:memory" class="w-[14px] h-[14px] text-[var(--holo-purple)] transition-colors duration-200" />
    <span
      class="font-medium tabular-nums min-w-[3.5ch] text-right transition-colors duration-200"
      :class="statusColor"
    >{{ usageText }}</span>
  </div>
</template>
