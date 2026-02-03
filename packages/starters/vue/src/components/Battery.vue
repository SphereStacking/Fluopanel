<script setup lang="ts">
import { computed } from 'vue'
import { Icon } from '@iconify/vue'
import { useBatteryProvider } from 'fluopanel-vue'

interface Props {
  direction?: 'horizontal' | 'vertical'
}
const props = withDefaults(defineProps<Props>(), {
  direction: 'horizontal',
})
const isVertical = computed(() => props.direction === 'vertical')

const { data: battery } = useBatteryProvider()

const icon = computed(() => {
  if (!battery.value) return 'mdi:battery'
  if (battery.value.charging) return 'mdi:battery-charging'
  const percent = battery.value.percent
  if (percent > 90) return 'mdi:battery'
  if (percent > 80) return 'mdi:battery-90'
  if (percent > 70) return 'mdi:battery-80'
  if (percent > 60) return 'mdi:battery-70'
  if (percent > 50) return 'mdi:battery-60'
  if (percent > 40) return 'mdi:battery-50'
  if (percent > 30) return 'mdi:battery-40'
  if (percent > 20) return 'mdi:battery-30'
  if (percent > 10) return 'mdi:battery-20'
  return 'mdi:battery-10'
})

const percentText = computed(() => {
  if (!battery.value) return '—'
  return `${Math.round(battery.value.percent)}%`
})

const statusColor = computed(() => {
  if (!battery.value) return 'text-[var(--text-secondary)]'
  if (battery.value.charging) return 'text-[var(--holo-cyan)]'
  if (battery.value.percent <= 20) return 'text-[var(--danger)]'
  if (battery.value.percent <= 40) return 'text-[var(--warning)]'
  return 'text-[var(--text-secondary)]'
})
</script>

<template>
  <!-- Horizontal: icon + text -->
  <figure
    v-if="battery && !isVertical"
    class="
      flex items-center gap-1.5 py-1 px-2.5 rounded-lg
      text-[12px] tracking-wide
      transition-all duration-200
      hover:bg-[var(--widget-glass-hover)]
      cursor-default
      group
    "
  >
    <Icon
      :icon="icon"
      class="w-[14px] h-[14px] transition-colors duration-200"
      :class="[statusColor, { 'animate-pulse': battery.charging }]"
    />
    <figcaption
      class="font-medium tabular-nums min-w-[3ch] text-right text-[var(--text-secondary)] group-hover:text-[var(--text-primary)] transition-colors duration-200"
    >{{ percentText }}</figcaption>
  </figure>

  <!-- Vertical: icon only with title -->
  <button
    v-else-if="battery && isVertical"
    type="button"
    class="p-1.5 rounded-lg hover:bg-[var(--widget-glass-hover)] transition-colors"
    :title="`Battery: ${percentText}${battery.charging ? ' (Charging)' : ''}`"
  >
    <Icon
      :icon="icon"
      class="w-4 h-4 transition-colors duration-200"
      :class="[statusColor, { 'animate-pulse': battery.charging }]"
    />
  </button>
</template>
