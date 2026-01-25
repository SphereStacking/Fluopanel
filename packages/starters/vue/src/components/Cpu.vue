<script setup lang="ts">
import { computed } from 'vue'
import { Icon } from '@iconify/vue'
import { useCpuProvider } from '@arcana/vue'

interface Props {
  direction?: 'horizontal' | 'vertical'
}
const props = withDefaults(defineProps<Props>(), {
  direction: 'horizontal',
})
const isVertical = computed(() => props.direction === 'vertical')

const { data: cpu } = useCpuProvider()

const usageText = computed(() => {
  if (!cpu.value) return '—'
  return `${Math.round(cpu.value.usage)}%`
})

const statusColor = computed(() => {
  if (!cpu.value) return 'text-[var(--text-secondary)]'
  if (cpu.value.usage > 80) return 'text-[var(--danger)]'
  if (cpu.value.usage > 50) return 'text-[var(--warning)]'
  return 'text-[var(--holo-blue)]'
})

const glowClass = computed(() => {
  if (!cpu.value) return ''
  if (cpu.value.usage > 80) return 'shadow-[0_0_8px_rgba(248,113,113,0.4)]'
  return ''
})
</script>

<template>
  <!-- Horizontal: icon + text -->
  <figure
    v-if="cpu && !isVertical"
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
    <Icon icon="mdi:chip" class="w-[14px] h-[14px] text-[var(--holo-blue)] transition-colors duration-200" />
    <figcaption
      class="font-medium tabular-nums min-w-[2.5ch] text-right transition-colors duration-200"
      :class="statusColor"
    >{{ usageText }}</figcaption>
  </figure>

  <!-- Vertical: icon only with title -->
  <button
    v-else-if="cpu && isVertical"
    type="button"
    class="p-1.5 rounded-lg hover:bg-[var(--widget-glass-hover)] transition-colors"
    :class="glowClass"
    :title="`CPU: ${usageText}`"
  >
    <Icon icon="mdi:chip" class="w-4 h-4" :class="statusColor" />
  </button>
</template>
