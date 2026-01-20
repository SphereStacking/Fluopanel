<script setup lang="ts">
import { computed } from 'vue'
import { isPopup, getPopupId } from '@arcana/core'

const props = defineProps<{
  /** Popup identifier (must match the ID used in usePopup.toggle()) */
  id: string
}>()

const currentPopupId = getPopupId()
const isPopupMode = isPopup()

// Render content only when:
// 1. Current window is a popup (has ?popup= parameter)
// 2. The popup ID matches this component's ID
const shouldRenderContent = computed(() => {
  return isPopupMode && currentPopupId === props.id
})
</script>

<template>
  <!-- Only render slot content when this is the active popup -->
  <slot v-if="shouldRenderContent" />
</template>
