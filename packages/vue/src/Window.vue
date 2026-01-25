<script setup lang="ts">
import { onMounted, onUnmounted, watch, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import {
  createInlineWindow,
  closeInlineWindow,
  updateWindowPosition,
  isCoordinator,
  getWindowId,
  type WindowPosition,
  type WindowConfig,
} from '@arcana/core'
import { registerPendingWindow, markWindowCompleted } from './composables/window-registry'

const props = defineProps<{
  /** Unique window identifier */
  id: string
  /** Window positioning (bounding box) */
  position: WindowPosition
  /** Window configuration */
  window?: WindowConfig
  /** Custom URL for external widgets (optional) */
  url?: string
}>()

const currentWindowId = getWindowId()
const isCoordinatorMode = isCoordinator()

// Check if this Window should render its content (when running as this window)
const shouldRenderContent = computed(() => {
  return !isCoordinatorMode && currentWindowId === props.id
})

// Check if this Window should create a window (when running as coordinator)
const shouldCreateWindow = computed(() => {
  return isCoordinatorMode
})

// Register this window as pending if in coordinator mode
if (shouldCreateWindow.value) {
  registerPendingWindow(props.id)
}

let unlisten: UnlistenFn | null = null

async function createWindow() {
  try {
    await createInlineWindow({
      id: props.id,
      position: props.position,
      window: props.window,
      url: props.url,
    })
  } catch {
    // Window might already exist (hot reload), update position instead
    await updateWindowPosition(props.id, props.position)
  }
}

async function destroyWindow() {
  try {
    await closeInlineWindow(props.id)
  } catch {
    // Window might not exist, ignore
  }
}

onMounted(async () => {
  if (shouldCreateWindow.value) {
    await createWindow()

    // Mark window as completed in registry
    markWindowCompleted(props.id)

    // Subscribe to monitor changes for repositioning
    unlisten = await listen('monitor-changed', async () => {
      await updateWindowPosition(props.id, props.position)
    })
  }
})

onUnmounted(async () => {
  unlisten?.()
  if (shouldCreateWindow.value) {
    await destroyWindow()
  }
})

// Watch for position changes and update window
watch(
  () => props.position,
  async (newPosition) => {
    if (shouldCreateWindow.value) {
      await updateWindowPosition(props.id, newPosition)
    }
  },
  { deep: true }
)
</script>

<template>
  <!-- Coordinator mode: don't render anything (window is created via Tauri) -->
  <!-- Window mode: render slot content -->
  <slot v-if="shouldRenderContent" />
</template>
