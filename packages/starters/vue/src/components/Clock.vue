<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import type { DateProvider, DateInfo } from '@arcana/providers'

const props = defineProps<{
  provider: DateProvider
}>()

const date = ref<DateInfo>({ timestamp: 0, formatted: '' })
let unsubscribe: (() => void) | null = null

onMounted(() => {
  date.value = props.provider.getDate('HH:mm')
  unsubscribe = props.provider.startPolling((info) => {
    date.value = info
  }, 1000)
})

onUnmounted(() => {
  unsubscribe?.()
})

// Split time for styling
const timeParts = computed(() => {
  const parts = date.value.formatted.split(':')
  return {
    hours: parts[0] || '00',
    minutes: parts[1] || '00'
  }
})

// Get day info
const dayInfo = computed(() => {
  const d = new Date(date.value.timestamp || Date.now())
  const days = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']
  const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']
  return {
    day: days[d.getDay()],
    date: d.getDate(),
    month: months[d.getMonth()]
  }
})
</script>

<template>
  <div
    class="
      flex items-center gap-3 py-1 px-2
      rounded-lg
      cursor-default
      group
    "
  >
    <!-- Time display with holographic accent -->
    <div class="flex items-baseline gap-0.5">
      <span
        class="
          text-[18px] font-bold tracking-tight
          text-[var(--text-primary)]
          transition-all duration-300
          group-hover:holo-text
        "
      >{{ timeParts.hours }}</span>
      <span
        class="
          text-[18px] font-light
          text-[var(--holo-cyan)]
          animate-pulse
        "
      >:</span>
      <span
        class="
          text-[18px] font-bold tracking-tight
          text-[var(--text-primary)]
          transition-all duration-300
          group-hover:holo-text
        "
      >{{ timeParts.minutes }}</span>
    </div>

    <!-- Date info -->
    <div class="flex flex-col items-start -space-y-0.5">
      <span class="text-[10px] font-medium text-[var(--text-tertiary)] uppercase tracking-wider">
        {{ dayInfo.day }}
      </span>
      <span class="text-[11px] font-semibold text-[var(--text-secondary)]">
        {{ dayInfo.month }} {{ dayInfo.date }}
      </span>
    </div>
  </div>
</template>
