<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { Icon } from '@iconify/vue'
import type { VolumeProvider, VolumeInfo } from '@arcana/providers'

const props = defineProps<{
  provider: VolumeProvider
}>()

const volume = ref<VolumeInfo | null>(null)
let intervalId: ReturnType<typeof setInterval> | null = null

const refresh = async () => {
  try {
    volume.value = await props.provider.getVolume()
  } catch (error) {
    console.error('Failed to get volume info:', error)
  }
}

onMounted(() => {
  refresh()
  intervalId = setInterval(refresh, 2000)
})

onUnmounted(() => {
  if (intervalId) clearInterval(intervalId)
})

const icon = computed(() => {
  if (!volume.value) return 'mdi:volume-high'
  if (volume.value.muted) return 'mdi:volume-off'
  if (volume.value.volume > 66) return 'mdi:volume-high'
  if (volume.value.volume > 33) return 'mdi:volume-medium'
  if (volume.value.volume > 0) return 'mdi:volume-low'
  return 'mdi:volume-off'
})

const volumeText = computed(() => {
  if (!volume.value) return '—'
  if (volume.value.muted) return 'Muted'
  return `${Math.round(volume.value.volume)}%`
})

const statusColor = computed(() => {
  if (!volume.value) return 'text-[var(--text-secondary)]'
  if (volume.value.muted) return 'text-[var(--text-ghost)]'
  return 'text-[var(--holo-pink)]'
})

const toggleMute = async () => {
  try {
    await props.provider.toggleMute()
    await refresh()
  } catch (error) {
    console.error('Failed to toggle mute:', error)
  }
}
</script>

<template>
  <div
    v-if="volume"
    class="
      flex items-center gap-1.5 py-1 px-2.5 rounded-lg
      text-[12px] tracking-wide
      transition-all duration-200
      hover:bg-[var(--widget-glass-hover)]
      cursor-pointer
      group
    "
    @click="toggleMute"
  >
    <Icon
      :icon="icon"
      class="w-[14px] h-[14px] transition-colors duration-200"
      :class="statusColor"
    />
    <span
      class="
        font-medium tabular-nums min-w-[3.5ch] text-right
        transition-colors duration-200
      "
      :class="volume.muted ? 'text-[var(--text-ghost)]' : 'text-[var(--text-secondary)] group-hover:text-[var(--text-primary)]'"
    >{{ volumeText }}</span>
  </div>
</template>
