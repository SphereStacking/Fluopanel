<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { Icon } from '@iconify/vue'
import type { MediaProvider, MediaInfo } from '@arcana/providers'

const props = defineProps<{
  provider: MediaProvider
}>()

const media = ref<MediaInfo | null>(null)
let intervalId: ReturnType<typeof setInterval> | null = null

const refresh = async () => {
  try {
    media.value = await props.provider.getMedia()
  } catch (error) {
    console.error('Failed to get media info:', error)
  }
}

onMounted(() => {
  refresh()
  intervalId = setInterval(refresh, 1000)
})

onUnmounted(() => {
  if (intervalId) clearInterval(intervalId)
})

const icon = computed(() => {
  if (!media.value) return 'mdi:music-note'
  return media.value.playing ? 'mdi:pause' : 'mdi:play'
})

const displayText = computed(() => {
  if (!media.value?.title) return null
  const artist = media.value.artist ? ` - ${media.value.artist}` : ''
  const text = `${media.value.title}${artist}`
  return text.length > 28 ? text.slice(0, 28) + '...' : text
})

const togglePlay = async () => {
  if (!media.value) return
  try {
    if (media.value.playing) {
      await props.provider.pause()
    } else {
      await props.provider.play()
    }
    await refresh()
  } catch (error) {
    console.error('Failed to toggle play:', error)
  }
}

const next = async () => {
  try {
    await props.provider.next()
    await refresh()
  } catch (error) {
    console.error('Failed to skip:', error)
  }
}

const prev = async () => {
  try {
    await props.provider.previous()
    await refresh()
  } catch (error) {
    console.error('Failed to go back:', error)
  }
}
</script>

<template>
  <div
    v-if="media?.title"
    class="
      flex items-center gap-2 py-1 px-3
      rounded-xl glass-widget
      group
    "
  >
    <!-- Previous button -->
    <button
      @click="prev"
      class="
        text-[var(--text-tertiary)]
        hover:text-[var(--holo-pink)]
        transition-all duration-200
        hover:scale-110
        focus:outline-none
      "
    >
      <Icon icon="mdi:skip-previous" class="w-[14px] h-[14px]" />
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
      :class="media.playing
        ? 'text-[var(--holo-cyan)] drop-shadow-[0_0_8px_var(--accent-glow)]'
        : 'text-[var(--text-secondary)] hover:text-[var(--holo-cyan)]'"
    >
      <Icon :icon="icon" class="w-[18px] h-[18px]" />
    </button>

    <!-- Next button -->
    <button
      @click="next"
      class="
        text-[var(--text-tertiary)]
        hover:text-[var(--holo-pink)]
        transition-all duration-200
        hover:scale-110
        focus:outline-none
      "
    >
      <Icon icon="mdi:skip-next" class="w-[14px] h-[14px]" />
    </button>

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
  </div>
</template>
