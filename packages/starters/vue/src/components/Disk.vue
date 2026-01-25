<script setup lang="ts">
import { computed } from 'vue'
import { Icon } from '@iconify/vue'
import { useDiskProvider } from '@arcana/vue'

interface Props {
  direction?: 'horizontal' | 'vertical'
}
const props = withDefaults(defineProps<Props>(), {
  direction: 'horizontal',
})
const isVertical = computed(() => props.direction === 'vertical')

const { data: disk } = useDiskProvider('/')

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
  <!-- Horizontal: icon + text -->
  <figure
    v-if="disk && !isVertical"
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
    <figcaption
      class="font-medium tabular-nums min-w-[3.5ch] text-right transition-colors duration-200"
      :class="statusColor"
    >{{ availableText }}</figcaption>
  </figure>

  <!-- Vertical: icon only with title -->
  <button
    v-else-if="disk && isVertical"
    type="button"
    class="p-1.5 rounded-lg hover:bg-[var(--widget-glass-hover)] transition-colors"
    :class="glowClass"
    :title="`Disk: ${availableText} available`"
  >
    <Icon icon="mdi:harddisk" class="w-4 h-4" :class="statusColor" />
  </button>
</template>
