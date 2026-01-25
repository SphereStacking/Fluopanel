<script setup lang="ts">
import { computed } from 'vue'
import { Icon } from '@iconify/vue'
import { useBatteryProvider, useCpuProvider, useMemoryProvider, useDiskProvider } from '@arcana/vue'

// Providers
const { data: battery } = useBatteryProvider()
const { data: cpu } = useCpuProvider()
const { data: memory } = useMemoryProvider()
const { data: disk } = useDiskProvider()

// Battery icon
const batteryIcon = computed(() => {
  if (!battery.value) return 'mdi:battery'
  if (battery.value.charging) return 'mdi:battery-charging'
  const percent = battery.value.percent
  if (percent > 80) return 'mdi:battery'
  if (percent > 60) return 'mdi:battery-70'
  if (percent > 40) return 'mdi:battery-50'
  if (percent > 20) return 'mdi:battery-30'
  return 'mdi:battery-10'
})

const batteryColor = computed(() => {
  if (!battery.value) return 'text-[var(--text-secondary)]'
  if (battery.value.charging) return 'text-[var(--holo-cyan)]'
  if (battery.value.percent <= 20) return 'text-[var(--danger)]'
  return 'text-[var(--text-secondary)]'
})

// CPU usage color
const cpuColor = computed(() => {
  if (!cpu.value) return 'text-[var(--text-secondary)]'
  if (cpu.value.usage > 80) return 'text-[var(--danger)]'
  if (cpu.value.usage > 50) return 'text-[var(--warning)]'
  return 'text-[var(--text-secondary)]'
})

// Memory usage color
const memoryColor = computed(() => {
  if (!memory.value) return 'text-[var(--text-secondary)]'
  const usedPercent = (memory.value.used / memory.value.total) * 100
  if (usedPercent > 80) return 'text-[var(--danger)]'
  if (usedPercent > 60) return 'text-[var(--warning)]'
  return 'text-[var(--text-secondary)]'
})

// Disk usage color
const diskColor = computed(() => {
  if (!disk.value) return 'text-[var(--text-secondary)]'
  const usedPercent = (disk.value.used / disk.value.total) * 100
  if (usedPercent > 90) return 'text-[var(--danger)]'
  if (usedPercent > 70) return 'text-[var(--warning)]'
  return 'text-[var(--text-secondary)]'
})
</script>

<template>
  <!-- Vertical bar container with holographic glass effect -->
  <div
    class="
      relative flex flex-col items-center h-full w-10 py-3
      glass rounded-2xl
      overflow-hidden
    "
  >
    <!-- Holographic shimmer overlay (vertical) -->
    <div
      class="
        absolute inset-0 pointer-events-none
        bg-[linear-gradient(195deg,transparent_40%,rgba(255,255,255,0.03)_45%,rgba(255,255,255,0.05)_50%,rgba(255,255,255,0.03)_55%,transparent_60%)]
        bg-[length:100%_200%]
        animate-[shimmer-vertical_8s_ease-in-out_infinite]
      "
    />

    <!-- Subtle holographic edge glow -->
    <div
      class="
        absolute inset-0 pointer-events-none rounded-2xl
        bg-[radial-gradient(ellipse_50%_80%_at_-20%_50%,rgba(110,180,255,0.08),transparent)]
      "
    />

    <!-- System indicators (icons only) -->
    <aside class="flex flex-col items-center gap-2 z-10">
      <button
        v-if="disk"
        type="button"
        class="p-1.5 rounded-lg hover:bg-[var(--widget-glass-hover)] transition-colors"
        :title="`Disk: ${Math.round((disk.used / disk.total) * 100)}%`"
      >
        <Icon icon="mdi:harddisk" class="w-4 h-4" :class="diskColor" />
      </button>

      <button
        v-if="cpu"
        type="button"
        class="p-1.5 rounded-lg hover:bg-[var(--widget-glass-hover)] transition-colors"
        :title="`CPU: ${Math.round(cpu.usage)}%`"
      >
        <Icon icon="mdi:chip" class="w-4 h-4" :class="cpuColor" />
      </button>

      <button
        v-if="memory"
        type="button"
        class="p-1.5 rounded-lg hover:bg-[var(--widget-glass-hover)] transition-colors"
        :title="`Memory: ${Math.round((memory.used / memory.total) * 100)}%`"
      >
        <Icon icon="mdi:memory" class="w-4 h-4" :class="memoryColor" />
      </button>

      <button
        v-if="battery"
        type="button"
        class="p-1.5 rounded-lg hover:bg-[var(--widget-glass-hover)] transition-colors"
        :title="`Battery: ${Math.round(battery.percent)}%${battery.charging ? ' (Charging)' : ''}`"
      >
        <Icon :icon="batteryIcon" class="w-4 h-4" :class="batteryColor" />
      </button>
    </aside>
  </div>
</template>

<style scoped>
@keyframes shimmer-vertical {
  0% { background-position: 0 200%; }
  100% { background-position: 0 -200%; }
}
</style>
