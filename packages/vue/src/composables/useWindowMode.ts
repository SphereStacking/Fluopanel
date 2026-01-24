import { onMounted, ref } from 'vue'
import {
  getWindowContext,
  isCoordinator as coreIsCoordinator,
  isWindow as coreIsWindow,
  getWindowId as coreGetWindowId,
  hideCoordinatorWindow,
  type WindowContext,
} from '@arcana/core'
import { waitForAllWindows } from './window-registry'

/**
 * Vue composable for window mode detection and coordinator management.
 * Use this to determine if the current window is a coordinator or window.
 */
export function useWindowMode() {
  const context = ref<WindowContext>(getWindowContext())
  const isCoordinator = ref(coreIsCoordinator())
  const isWindow = ref(coreIsWindow())
  const windowId = ref<string | null>(coreGetWindowId())

  /**
   * Hide the coordinator window.
   * Call this in App.vue after all Window components are mounted.
   */
  async function hideCoordinator(): Promise<void> {
    if (isCoordinator.value) {
      await hideCoordinatorWindow()
    }
  }

  return {
    context,
    isCoordinator,
    isWindow,
    windowId,
    hideCoordinator,
  }
}

/**
 * Composable for coordinator mode.
 * Automatically hides the coordinator window after windows are created.
 */
export function useCoordinator(options?: { autoHide?: boolean }) {
  const { autoHide = true } = options ?? {}
  const { isCoordinator, hideCoordinator } = useWindowMode()

  onMounted(async () => {
    if (isCoordinator.value && autoHide) {
      // Wait for all windows to be created first
      await waitForAllWindows()
      await hideCoordinator()
    }
  })

  return { isCoordinator }
}
