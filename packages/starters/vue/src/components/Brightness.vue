<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { Icon } from '@iconify/vue'
import type { BrightnessProvider, BrightnessInfo } from '@arcana/providers'

const props = defineProps<{
  provider: BrightnessProvider
}>()

const brightness = ref<BrightnessInfo | null>(null)
let unsubscribe: (() => void) | null = null

const refresh = async () => {
  try {
    brightness.value = await props.provider.getBrightness()
  } catch (error) {
    console.error('Failed to get brightness info:', error)
  }
}

onMounted(async () => {
  await refresh()
  unsubscribe = props.provider.onBrightnessChange((info) => {
    brightness.value = info
  })
})

onUnmounted(() => {
  unsubscribe?.()
})

const icon = computed(() => {
  if (!brightness.value) return 'mdi:brightness-6'
  const level = brightness.value.brightness
  if (level > 0.75) return 'mdi:brightness-7'
  if (level > 0.5) return 'mdi:brightness-6'
  if (level > 0.25) return 'mdi:brightness-5'
  return 'mdi:brightness-4'
})

const brightnessText = computed(() => {
  if (!brightness.value) return '—'
  return `${Math.round(brightness.value.brightness * 100)}%`
})
</script>

<template>
  <div
    v-if="brightness"
    class="
      flex items-center gap-1.5 py-1 px-2.5 rounded-lg
      text-[12px] tracking-wide
      transition-all duration-200
      hover:bg-[var(--widget-glass-hover)]
      cursor-default
      group
    "
  >
    <Icon :icon="icon" class="w-[14px] h-[14px] text-[var(--holo-yellow)] transition-colors duration-200" />
    <span
      class="font-medium tabular-nums min-w-[3ch] text-right text-[var(--text-secondary)] group-hover:text-[var(--text-primary)] transition-colors duration-200"
    >{{ brightnessText }}</span>
  </div>
</template>
