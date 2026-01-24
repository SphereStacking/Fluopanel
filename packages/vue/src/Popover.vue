<script setup lang="ts">
import { computed } from 'vue'
import { getPopoverContext } from '@arcana/core'

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
</script>

<template>
  <!-- Only render slot content when this is the active popover -->
  <slot v-if="shouldRenderContent" />
</template>
