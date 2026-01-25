<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { getPopoverContext } from '@arcana/core'
import { useAutoSize } from './composables/useAutoSize'

const props = defineProps<{
  /** Popover identifier (must match the ID used in usePopover.toggle()) */
  id: string
}>()

// Get popover context once at setup (URL won't change during component lifecycle)
const popoverContext = getPopoverContext()

// Render content only when:
// 1. Current window is a popover (has ?popover= parameter)
// 2. The popover ID matches this component's ID
const shouldRenderContent = computed(() => {
  return popoverContext.isPopover && popoverContext.id === props.id
})

// Animation key: incremented on popover-reopen to force slot re-render
// This replays animations without page reload (avoiding flicker)
const animationKey = ref(0)

onMounted(() => {
  if (shouldRenderContent.value) {
    const handleReopen = () => {
      animationKey.value++
    }
    window.addEventListener('popover-reopen', handleReopen)
    onUnmounted(() => {
      window.removeEventListener('popover-reopen', handleReopen)
    })
  }
})

// Auto-size the popover window based on content
// maxHeight is calculated by Rust side based on anchor position and screen bounds
const contentRef = ref<HTMLElement | null>(null)
const { height } = useAutoSize(contentRef, {
  enabled: shouldRenderContent,
  maxHeight: popoverContext.maxHeight ?? undefined,
})
</script>

<template>
  <!-- Only render slot content when this is the active popover -->
  <div
    v-if="shouldRenderContent"
    ref="contentRef"
    :style="{
      width: 'fit-content',
      height: height ? `${height}px` : 'fit-content',
      maxHeight: popoverContext.maxHeight ? `${popoverContext.maxHeight}px` : undefined,
      overflow: 'hidden',
    }"
  >
    <!-- Key changes on reopen to force slot re-render and replay animations -->
    <div :key="animationKey">
      <slot />
    </div>
  </div>
</template>

