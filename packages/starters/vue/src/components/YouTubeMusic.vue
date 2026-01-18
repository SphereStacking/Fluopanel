<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useYouTubeMusicProvider } from '@arcana/vue'

const { data: music, toggle, next, previous, seek } = useYouTubeMusicProvider()

let positionInterval: ReturnType<typeof setInterval> | null = null

// Local position tracking (update progress bar between polling intervals)
const localPosition = ref<number>(0)
// Seeking flag (stop auto-update during drag)
const isSeeking = ref(false)

// Sync local position with provider data
watch(music, (newMusic) => {
  if (newMusic?.position !== undefined && !isSeeking.value) {
    localPosition.value = newMusic.position
  }
}, { immediate: true })

onMounted(() => {
  // Update position every second while playing (stop during seek)
  positionInterval = setInterval(() => {
    if (music.value?.playing && music.value.duration && !isSeeking.value) {
      localPosition.value = Math.min(localPosition.value + 1, music.value.duration)
    }
  }, 1000)
})

onUnmounted(() => {
  if (positionInterval) clearInterval(positionInterval)
})

const icon = computed(() => {
  if (!music.value) return 'mdi:music-note'
  return music.value.playing ? 'mdi:pause' : 'mdi:play'
})

const displayText = computed(() => {
  if (!music.value?.title) return null
  const artist = music.value.artist ? ` - ${music.value.artist}` : ''
  return `${music.value.title}${artist}`
})

// Enable marquee scroll for long text
const needsMarquee = computed(() => {
  return displayText.value ? displayText.value.length > 20 : false
})



// Progress percentage (0-100)
const progress = computed(() => {
  if (!music.value?.duration || music.value.duration === 0) return 0
  return Math.min((localPosition.value / music.value.duration) * 100, 100)
})

const togglePlay = async () => {
  if (!music.value) return

  // Optimistic update: toggle UI immediately
  const wasPlaying = music.value.playing
  music.value = { ...music.value, playing: !wasPlaying }

  try {
    await toggle()
  } catch (error) {
    console.error('Failed to toggle play:', error)
    // Revert on error
    if (music.value) {
      music.value = { ...music.value, playing: wasPlaying }
    }
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

// Seek handlers
const onSeekInput = (e: Event) => {
  isSeeking.value = true
  localPosition.value = Number((e.target as HTMLInputElement).value)
}

const onSeekChange = async (e: Event) => {
  const seconds = Number((e.target as HTMLInputElement).value)
  try {
    await seek(seconds)
  } catch (error) {
    console.error('Failed to seek:', error)
  }
  isSeeking.value = false
}
</script>

<template>
  <div
    v-if="music?.title"
    class="
      flex items-center gap-2 py-1.5 px-3
      rounded-xl
      group
    "
  >
    <!-- Album Artwork -->
    <div class="relative shrink-0">
      <div
        class="
          w-7 h-7 rounded-lg overflow-hidden
          bg-[var(--glass-bg)]
          ring-1 ring-[var(--glass-border)]
          transition-all duration-300
          group-hover:ring-[var(--danger)]/40
          group-hover:shadow-[0_0_12px_rgba(248,113,113,0.2)]
        "
      >
        <img
          v-if="music.artworkUrl"
          :src="music.artworkUrl"
          :alt="music.title"
          class="
            w-full h-full object-cover
            transition-transform duration-300
            group-hover:scale-110
          "
        />
        <div
          v-else
          class="w-full h-full flex items-center justify-center"
        >
          <Icon
            icon="simple-icons:youtubemusic"
            class="w-4 h-4 text-[var(--danger)]/60"
          />
        </div>
      </div>
      <!-- Playing indicator -->
      <div
        v-if="music.playing"
        class="
          absolute -bottom-0.5 -right-0.5
          w-2 h-2 rounded-full
          bg-[var(--danger)]
          shadow-[0_0_6px_rgba(248,113,113,0.6)]
          animate-pulse
        "
      />
    </div>

    <!-- Controls -->
    <div class="flex items-center gap-1">
      <!-- Previous -->
      <button
        @click="handlePrev"
        class="
          p-0.5
          text-[var(--text-tertiary)]
          hover:text-[var(--danger)]
          transition-all duration-200
          hover:scale-110
          focus:outline-none
        "
      >
        <Icon icon="mdi:skip-previous" class="w-[14px] h-[14px]" />
      </button>

      <!-- Play/Pause -->
      <button
        @click="togglePlay"
        class="
          p-0.5
          transition-all duration-200
          hover:scale-110
          focus:outline-none
        "
        :class="music.playing
          ? 'text-[var(--danger)] drop-shadow-[0_0_8px_rgba(248,113,113,0.5)]'
          : 'text-[var(--text-secondary)] hover:text-[var(--danger)]'"
      >
        <Icon :icon="icon" class="w-[16px] h-[16px]" />
      </button>

      <!-- Next -->
      <button
        @click="handleNext"
        class="
          p-0.5
          text-[var(--text-tertiary)]
          hover:text-[var(--danger)]
          transition-all duration-200
          hover:scale-110
          focus:outline-none
        "
      >
        <Icon icon="mdi:skip-next" class="w-[14px] h-[14px]" />
      </button>
    </div>

    <!-- Divider -->
    <div class="w-px h-5 bg-[var(--glass-border)]" />

    <!-- Track Info & Progress -->
    <div class="flex flex-col gap-0.5 min-w-0 max-w-[160px]">
      <!-- Title & Artist with marquee -->
      <div class="overflow-hidden marquee-container">
        <div
          class="
            inline-flex whitespace-nowrap
            text-[11px] font-medium leading-tight
            text-[var(--text-secondary)]
            transition-colors duration-200
            group-hover:text-[var(--text-primary)]
          "
          :class="needsMarquee ? 'will-change-transform group-hover:animate-[marquee_6s_linear_infinite]' : ''"
        >
          <span>{{ displayText }}</span>
          <span v-if="needsMarquee" class="ml-8">{{ displayText }}</span>
        </div>
      </div>

      <!-- Progress Slider -->
      <input
        type="range"
        :min="0"
        :max="music.duration || 0"
        :value="localPosition"
        @input="onSeekInput"
        @change="onSeekChange"
        class="
          w-full h-[3px] appearance-none cursor-pointer
          bg-[var(--glass-border)] rounded-full
          [&::-webkit-slider-thumb]:appearance-none
          [&::-webkit-slider-thumb]:w-2 [&::-webkit-slider-thumb]:h-2
          [&::-webkit-slider-thumb]:rounded-full
          [&::-webkit-slider-thumb]:bg-[var(--danger)]
          [&::-webkit-slider-thumb]:shadow-[0_0_4px_rgba(248,113,113,0.5)]
          [&::-webkit-slider-thumb]:transition-transform
          [&::-webkit-slider-thumb]:duration-150
          [&::-webkit-slider-thumb]:hover:scale-125
          [&::-webkit-slider-thumb]:-mt-[2.5px]
          [&::-webkit-slider-runnable-track]:h-[3px]
          [&::-webkit-slider-runnable-track]:rounded-full
        "
        :style="{
          background: `linear-gradient(to right, #f87171 0%, #fca5a5 ${progress}%, var(--glass-border) ${progress}%)`
        }"
      />
    </div>
  </div>
</template>
