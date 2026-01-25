<script setup lang="ts">
import { ref, computed } from 'vue'
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

// Auto-size the popover window based on content
// maxHeight is calculated by Rust side based on anchor position and screen bounds
const contentRef = ref<HTMLElement | null>(null)
useAutoSize(contentRef, {
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
      height: 'fit-content',
      maxHeight: popoverContext.maxHeight ? `${popoverContext.maxHeight}px` : undefined,
      overflow: 'hidden',
    }"
  >
    <slot />
  </div>
</template>
