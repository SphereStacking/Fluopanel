<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Icon } from '@iconify/vue'
import { useVolumeProvider } from 'fluopanel-vue'

const { data: volume, setVolume, toggleMute } = useVolumeProvider()

const isHovered = ref(false)
const isDragging = ref(false)
const localVolume = ref(0)

// Sync local volume with provider data
watch(volume, (newVolume) => {
  if (newVolume && !isDragging.value) {
    localVolume.value = newVolume.volume
  }
}, { immediate: true })

const icon = computed(() => {
  if (!volume.value) return 'mdi:volume-high'
  if (volume.value.muted) return 'mdi:volume-off'
  const vol = isDragging.value ? localVolume.value : volume.value.volume
  if (vol > 66) return 'mdi:volume-high'
  if (vol > 33) return 'mdi:volume-medium'
  if (vol > 0) return 'mdi:volume-low'
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

const handleToggleMute = async () => {
  try {
    await toggleMute()
  } catch (error) {
    console.error('Failed to toggle mute:', error)
  }
}

const onVolumeInput = (e: Event) => {
  isDragging.value = true
  localVolume.value = Number((e.target as HTMLInputElement).value)
}

const onVolumeChange = async (e: Event) => {
  const value = Number((e.target as HTMLInputElement).value)
  try {
    await setVolume(value)
  } catch (error) {
    console.error('Failed to set volume:', error)
  }
  isDragging.value = false
}
</script>

<template>
  <figure
    v-if="volume"
    class="
      flex items-center gap-1.5 py-1 px-2.5 rounded-lg
      text-[12px] tracking-wide
      transition-all duration-200
      hover:bg-[var(--widget-glass-hover)]
      cursor-pointer
      group
    "
    @mouseenter="isHovered = true"
    @mouseleave="isHovered = false"
  >
    <!-- Icon (click to mute) -->
    <Icon
      :icon="icon"
      class="w-[14px] h-[14px] transition-colors duration-200 shrink-0"
      :class="statusColor"
      @click.stop="handleToggleMute"
    />

    <!-- Text/Slider container with width animation -->
    <div
      class="relative h-[14px] flex items-center overflow-hidden transition-all duration-200"
      :class="isHovered ? 'w-16' : 'w-[4ch]'"
    >
      <!-- Volume text (default) -->
      <span
        class="
          absolute right-0
          font-medium tabular-nums text-right
          transition-opacity duration-200
        "
        :class="[
          volume.muted ? 'text-[var(--text-ghost)]' : 'text-[var(--text-secondary)] group-hover:text-[var(--text-primary)]',
          isHovered ? 'opacity-0 pointer-events-none' : 'opacity-100'
        ]"
      >{{ volumeText }}</span>

      <!-- Volume slider (on hover) -->
      <input
        type="range"
        :min="0"
        :max="100"
        :value="localVolume"
        @input="onVolumeInput"
        @change="onVolumeChange"
        @click.stop
        class="
          absolute left-0
          w-16 h-[3px] appearance-none cursor-pointer
          bg-[var(--glass-border)] rounded-full
          transition-opacity duration-200
          [&::-webkit-slider-thumb]:appearance-none
          [&::-webkit-slider-thumb]:w-2 [&::-webkit-slider-thumb]:h-2
          [&::-webkit-slider-thumb]:rounded-full
          [&::-webkit-slider-thumb]:bg-[var(--holo-pink)]
          [&::-webkit-slider-thumb]:shadow-[0_0_4px_rgba(255,111,216,0.5)]
          [&::-webkit-slider-thumb]:transition-transform
          [&::-webkit-slider-thumb]:duration-150
          [&::-webkit-slider-thumb]:hover:scale-125
          [&::-webkit-slider-thumb]:-mt-[2.5px]
          [&::-webkit-slider-runnable-track]:h-[3px]
          [&::-webkit-slider-runnable-track]:rounded-full
        "
        :class="isHovered ? 'opacity-100' : 'opacity-0 pointer-events-none'"
        :style="{
          background: `linear-gradient(to right, var(--holo-pink) 0%, var(--holo-pink) ${localVolume}%, var(--glass-border) ${localVolume}%)`
        }"
      />
    </div>
  </figure>
</template>
