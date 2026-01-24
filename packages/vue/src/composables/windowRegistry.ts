import { ref, readonly } from 'vue'

/**
 * Window creation registry to track window creation completion.
 * Used to ensure all windows are created before hiding the coordinator.
 */

const pendingWindows = ref(new Set<string>())
const completedWindows = ref(new Set<string>())
const resolvers: (() => void)[] = []

/**
 * Register a window as pending creation.
 */
export function registerPendingWindow(id: string): void {
  pendingWindows.value.add(id)
}

/**
 * Mark a window as completed.
 */
export function markWindowCompleted(id: string): void {
  pendingWindows.value.delete(id)
  completedWindows.value.add(id)

  // Check if all windows are completed
  if (pendingWindows.value.size === 0 && resolvers.length > 0) {
    resolvers.forEach(resolve => resolve())
    resolvers.length = 0
  }
}

/**
 * Wait for all pending windows to complete.
 * Returns immediately if no windows are pending.
 */
export function waitForAllWindows(): Promise<void> {
  if (pendingWindows.value.size === 0) {
    return Promise.resolve()
  }

  return new Promise<void>((resolve) => {
    resolvers.push(resolve)
  })
}

/**
 * Reset the registry (useful for testing or hot reload).
 */
export function resetWindowRegistry(): void {
  pendingWindows.value.clear()
  completedWindows.value.clear()
  resolvers.length = 0
}

export const windowRegistryState = {
  pending: readonly(pendingWindows),
  completed: readonly(completedWindows),
}
