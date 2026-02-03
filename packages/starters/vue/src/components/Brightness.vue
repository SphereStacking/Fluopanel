<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Icon } from '@iconify/vue'
import { useBrightnessProvider } from 'fluopanel-vue'

const { data: brightness, setBrightness } = useBrightnessProvider()

const isHovered = ref(false)
const isDragging = ref(false)
const localBrightness = ref(0)

// Sync local brightness with provider data
watch(brightness, (newBrightness) => {
  if (newBrightness && !isDragging.value) {
    localBrightness.value = newBrightness.brightness
  }
}, { immediate: true })

const icon = computed(() => {
  if (!brightness.value) return 'mdi:brightness-6'
  const level = isDragging.value ? localBrightness.value : brightness.value.brightness
  if (level > 75) return 'mdi:brightness-7'
  if (level > 50) return 'mdi:brightness-6'
  if (level > 25) return 'mdi:brightness-5'
  return 'mdi:brightness-4'
})

const brightnessText = computed(() => {
  if (!brightness.value) return '—'
  return `${Math.round(brightness.value.brightness)}%`
})

const onBrightnessInput = (e: Event) => {
  isDragging.value = true
  localBrightness.value = Number((e.target as HTMLInputElement).value)
}

const onBrightnessChange = async (e: Event) => {
  const value = Number((e.target as HTMLInputElement).value)
  try {
    await setBrightness(value)
  } catch (error) {
    console.error('Failed to set brightness:', error)
  }
  isDragging.value = false
}
</script>

<template>
  <figure
    v-if="brightness"
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
    <!-- Icon -->
    <Icon
      :icon="icon"
      class="w-[14px] h-[14px] text-[var(--holo-yellow)] transition-colors duration-200 shrink-0"
    />

    <!-- Text/Slider container with width animation -->
    <div
      class="relative h-[14px] flex items-center overflow-hidden transition-all duration-200"
      :class="isHovered ? 'w-16' : 'w-[4ch]'"
    >
      <!-- Brightness text (default) -->
      <span
        class="
          absolute right-0
          font-medium tabular-nums text-right
          text-[var(--text-secondary)] group-hover:text-[var(--text-primary)]
          transition-opacity duration-200
        "
        :class="isHovered ? 'opacity-0 pointer-events-none' : 'opacity-100'"
      >{{ brightnessText }}</span>

      <!-- Brightness slider (on hover) -->
      <input
        type="range"
        :min="0"
        :max="100"
        :value="localBrightness"
        @input="onBrightnessInput"
        @change="onBrightnessChange"
        @click.stop
        class="
          absolute left-0
          w-16 h-[3px] appearance-none cursor-pointer
          bg-[var(--glass-border)] rounded-full
          transition-opacity duration-200
          [&::-webkit-slider-thumb]:appearance-none
          [&::-webkit-slider-thumb]:w-2 [&::-webkit-slider-thumb]:h-2
          [&::-webkit-slider-thumb]:rounded-full
          [&::-webkit-slider-thumb]:bg-[var(--holo-yellow)]
          [&::-webkit-slider-thumb]:shadow-[0_0_4px_rgba(253,230,138,0.5)]
          [&::-webkit-slider-thumb]:transition-transform
          [&::-webkit-slider-thumb]:duration-150
          [&::-webkit-slider-thumb]:hover:scale-125
          [&::-webkit-slider-thumb]:-mt-[2.5px]
          [&::-webkit-slider-runnable-track]:h-[3px]
          [&::-webkit-slider-runnable-track]:rounded-full
        "
        :class="isHovered ? 'opacity-100' : 'opacity-0 pointer-events-none'"
        :style="{
          background: `linear-gradient(to right, var(--holo-yellow) 0%, var(--holo-yellow) ${localBrightness}%, var(--glass-border) ${localBrightness}%)`
        }"
      />
    </div>
  </figure>
</template>
