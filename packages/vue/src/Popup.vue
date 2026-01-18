<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { isPopup, getPopupId, getPopupMode } from '@arcana/core'

const props = defineProps<{
  /** Popup identifier (must match the ID used in usePopup.open()) */
  id: string
}>()

const currentPopupId = getPopupId()
const isPopupMode = isPopup()
const popupMode = getPopupMode()

// Render content only when:
// 1. Current window is a popup (has ?popup= parameter)
// 2. The popup ID matches this component's ID
const shouldRenderContent = computed(() => {
  return isPopupMode && currentPopupId === props.id
})

// Mouse tracking for hover mode
function handleMouseEnter() {
  if (popupMode === 'hover' && currentPopupId) {
    invoke('popup_window_enter', { popupId: currentPopupId }).catch(console.error)
  }
}

function handleMouseLeave() {
  if (popupMode === 'hover' && currentPopupId) {
    invoke('popup_window_leave', { popupId: currentPopupId }).catch(console.error)
  }
}

// Set up document-level listeners for hover mode
// This ensures we capture mouse events even when hovering over child elements
let documentEnterHandler: (() => void) | null = null
let documentLeaveHandler: (() => void) | null = null

onMounted(() => {
  if (shouldRenderContent.value && popupMode === 'hover') {
    documentEnterHandler = handleMouseEnter
    documentLeaveHandler = handleMouseLeave

    document.documentElement.addEventListener('mouseenter', documentEnterHandler)
    document.documentElement.addEventListener('mouseleave', documentLeaveHandler)

    // Initial state: mouse is over popup when it opens (user hovered to open)
    handleMouseEnter()
  }
})

onUnmounted(() => {
  if (documentEnterHandler) {
    document.documentElement.removeEventListener('mouseenter', documentEnterHandler)
    documentEnterHandler = null
  }
  if (documentLeaveHandler) {
    document.documentElement.removeEventListener('mouseleave', documentLeaveHandler)
    documentLeaveHandler = null
  }
})
</script>

<template>
  <!-- Only render slot content when this is the active popup -->
  <slot v-if="shouldRenderContent" />
</template>
