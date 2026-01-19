<script setup lang="ts">
import { onMounted, onUnmounted, watch, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import {
  createInlineWidgetWindow,
  closeInlineWidgetWindow,
  updateWidgetPosition,
  isCoordinator,
  getWidgetId,
  type WidgetPosition,
  type WidgetWindowConfig,
} from '@arcana/core'
import { registerPendingWidget, markWidgetCompleted } from './composables/widgetRegistry'

const props = defineProps<{
  /** Unique widget identifier */
  id: string
  /** Widget positioning (bounding box) */
  position: WidgetPosition
  /** Window configuration */
  window?: WidgetWindowConfig
}>()

const currentWidgetId = getWidgetId()
const isCoordinatorMode = isCoordinator()

// Check if this Widget should render its content (when running as this widget)
const shouldRenderContent = computed(() => {
  return !isCoordinatorMode && currentWidgetId === props.id
})

// Check if this Widget should create a window (when running as coordinator)
const shouldCreateWindow = computed(() => {
  return isCoordinatorMode
})

// Register this widget as pending if in coordinator mode
if (shouldCreateWindow.value) {
  registerPendingWidget(props.id)
}

let unlisten: UnlistenFn | null = null

async function createWindow() {
  try {
    await createInlineWidgetWindow({
      id: props.id,
      position: props.position,
      window: props.window,
    })
  } catch {
    // Window might already exist (hot reload), update position instead
    await updateWidgetPosition(props.id, props.position)
  }
}

async function destroyWindow() {
  try {
    await closeInlineWidgetWindow(props.id)
  } catch {
    // Window might not exist, ignore
  }
}

onMounted(async () => {
  if (shouldCreateWindow.value) {
    await createWindow()

    // Mark widget as completed in registry
    markWidgetCompleted(props.id)

    // Subscribe to monitor changes for repositioning
    unlisten = await listen('monitor-changed', async () => {
      await updateWidgetPosition(props.id, props.position)
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
      await updateWidgetPosition(props.id, newPosition)
    }
  },
  { deep: true }
)
</script>

<template>
  <!-- Coordinator mode: don't render anything (window is created via Tauri) -->
  <!-- Widget mode: render slot content -->
  <slot v-if="shouldRenderContent" />
</template>
