<script setup lang="ts">
import { computed } from 'vue'
import { Icon } from '@iconify/vue'
import { useMediaProvider } from 'fluopanel-vue'

interface Props {
  direction?: 'horizontal' | 'vertical'
}
const props = withDefaults(defineProps<Props>(), {
  direction: 'horizontal'
})

const isVertical = computed(() => props.direction === 'vertical')

const { data: media, play, pause, next, previous } = useMediaProvider()

const icon = computed(() => {
  if (!media.value) return 'mdi:music-note'
  return media.value.playing ? 'mdi:pause' : 'mdi:play'
})

const displayText = computed(() => {
  if (!media.value?.title) return null
  const artist = media.value.artist ? ` - ${media.value.artist}` : ''
  const text = `${media.value.title}${artist}`
  // Only truncate for horizontal display
  if (!isVertical.value && text.length > 28) {
    return text.slice(0, 28) + '...'
  }
  return text
})

const togglePlay = async () => {
  if (!media.value) return
  try {
    if (media.value.playing) {
      await pause()
    } else {
      await play()
    }
  } catch (error) {
    console.error('Failed to toggle play:', error)
  }
}

const handleNext = async () => {
  try {
    await next()
  } catch (error) {
    console.error('Failed to skip:', error)
  }
}

const handlePrev = async () => {
  try {
    await previous()
  } catch (error) {
    console.error('Failed to go back:', error)
  }
}
</script>

<template>
  <component
    :is="isVertical ? 'div' : 'section'"
    v-if="media?.title"
    class="flex group"
    :class="isVertical
      ? 'flex-col items-center gap-0.5'
      : 'items-center gap-2 py-1 px-3 rounded-xl glass-widget'"
    :title="isVertical ? displayText || '' : undefined"
  >
    <!-- Previous button -->
    <button
      @click="handlePrev"
      class="
        text-[var(--text-tertiary)]
        hover:text-[var(--holo-pink)]
        transition-all duration-200
        hover:scale-110
        focus:outline-none
      "
      :class="isVertical ? 'p-1' : ''"
    >
      <Icon icon="mdi:skip-previous" :class="isVertical ? 'w-3.5 h-3.5' : 'w-[14px] h-[14px]'" />
    </button>

    <!-- Play/Pause button with glow -->
    <button
      @click="togglePlay"
      class="
        relative
        transition-all duration-200
        hover:scale-110
        focus:outline-none
      "
      :class="[
        media.playing
          ? 'text-[var(--holo-cyan)] drop-shadow-[0_0_8px_var(--accent-glow)]'
          : 'text-[var(--text-secondary)] hover:text-[var(--holo-cyan)]',
        isVertical ? 'p-1.5 rounded-lg hover:bg-[var(--widget-glass-hover)]' : ''
      ]"
    >
      <Icon :icon="icon" :class="isVertical ? 'w-5 h-5' : 'w-[18px] h-[18px]'" />
    </button>

    <!-- Next button -->
    <button
      @click="handleNext"
      class="
        text-[var(--text-tertiary)]
        hover:text-[var(--holo-pink)]
        transition-all duration-200
        hover:scale-110
        focus:outline-none
      "
      :class="isVertical ? 'p-1' : ''"
    >
      <Icon icon="mdi:skip-next" :class="isVertical ? 'w-3.5 h-3.5' : 'w-[14px] h-[14px]'" />
    </button>

    <!-- Track info - horizontal only -->
    <template v-if="!isVertical">
      <!-- Divider -->
      <div class="w-px h-4 bg-[var(--glass-border)] mx-1" />

      <!-- Track info with marquee effect potential -->
      <div class="flex items-center gap-2 max-w-[160px] overflow-hidden">
        <!-- Music note icon -->
        <Icon icon="mdi:music-note" class="w-3 h-3 text-[var(--holo-pink)] shrink-0" />

        <!-- Track title -->
        <span
          class="
            text-[12px] font-medium tracking-wide
            text-[var(--text-secondary)]
            truncate
            transition-colors duration-200
            group-hover:text-[var(--text-primary)]
          "
        >{{ displayText }}</span>
      </div>
    </template>
  </component>
</template>
