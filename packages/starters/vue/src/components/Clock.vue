<script setup lang="ts">
import { computed } from 'vue'
import { useDateProvider } from '@arcana/vue'

interface Props {
  direction?: 'horizontal' | 'vertical'
}
const props = withDefaults(defineProps<Props>(), {
  direction: 'horizontal'
})

const isVertical = computed(() => props.direction === 'vertical')

const { data: date } = useDateProvider('HH:mm', 60000)

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
  <time
    class="flex cursor-default group"
    :class="isVertical
      ? 'flex-col items-center py-1'
      : 'items-center gap-3 py-1 px-2 rounded-lg'"
    :title="isVertical ? `${dayInfo.day} ${dayInfo.date}` : undefined"
  >
    <!-- Vertical layout -->
    <template v-if="isVertical">
      <!-- Hours -->
      <span
        class="
          text-[14px] font-bold leading-tight tracking-tight
          text-[var(--text-primary)]
          transition-all duration-300
          group-hover:holo-text
        "
      >{{ timeParts.hours }}</span>

      <!-- Colon -->
      <span
        class="
          text-[10px] font-light leading-none
          text-[var(--holo-cyan)]
          animate-pulse
        "
      >:</span>

      <!-- Minutes -->
      <span
        class="
          text-[14px] font-bold leading-tight tracking-tight
          text-[var(--text-primary)]
          transition-all duration-300
          group-hover:holo-text
        "
      >{{ timeParts.minutes }}</span>

      <!-- Day -->
      <span
        class="
          text-[8px] font-medium mt-0.5
          text-[var(--text-tertiary)] uppercase tracking-wider
        "
      >{{ dayInfo.day }}</span>
    </template>

    <!-- Horizontal layout -->
    <template v-else>
      <!-- Time display with holographic accent -->
      <span class="flex items-baseline gap-0.5">
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
      </span>

      <!-- Date info -->
      <span class="flex flex-col items-start -space-y-0.5">
        <span class="text-[10px] font-medium text-[var(--text-tertiary)] uppercase tracking-wider">
          {{ dayInfo.day }}
        </span>
        <span class="text-[11px] font-semibold text-[var(--text-secondary)]">
          {{ dayInfo.month }} {{ dayInfo.date }}
        </span>
      </span>
    </template>
  </time>
</template>
