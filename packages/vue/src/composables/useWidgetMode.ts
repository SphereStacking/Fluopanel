import { onMounted, ref } from 'vue'
import {
  getWidgetContext,
  isCoordinator as coreIsCoordinator,
  isWidget as coreIsWidget,
  getWidgetId as coreGetWidgetId,
  hideCoordinatorWindow,
  type WidgetContext,
} from '@arcana/core'
import { waitForAllWidgets } from './widgetRegistry'

/**
 * Vue composable for widget mode detection and coordinator management.
 * Use this to determine if the current window is a coordinator or widget.
 */
export function useWidgetMode() {
  const context = ref<WidgetContext>(getWidgetContext())
  const isCoordinator = ref(coreIsCoordinator())
  const isWidget = ref(coreIsWidget())
  const widgetId = ref<string | null>(coreGetWidgetId())

  /**
   * Hide the coordinator window.
   * Call this in App.vue after all Widget components are mounted.
   */
  async function hideCoordinator(): Promise<void> {
    if (isCoordinator.value) {
      await hideCoordinatorWindow()
    }
  }

  return {
    context,
    isCoordinator,
    isWidget,
    widgetId,
    hideCoordinator,
  }
}

/**
 * Composable for coordinator mode.
 * Automatically hides the coordinator window after widgets are created.
 */
export function useCoordinator(options?: { autoHide?: boolean }) {
  const { autoHide = true } = options ?? {}
  const { isCoordinator, hideCoordinator } = useWidgetMode()

  onMounted(async () => {
    if (isCoordinator.value && autoHide) {
      // Wait for all widget windows to be created first
      await waitForAllWidgets()
      await hideCoordinator()
    }
  })

  return { isCoordinator }
}
