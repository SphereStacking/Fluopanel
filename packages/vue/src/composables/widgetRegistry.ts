import { ref, readonly } from 'vue'

/**
 * Widget creation registry to track widget window creation completion.
 * Used to ensure all widget windows are created before hiding the coordinator.
 */

const pendingWidgets = ref(new Set<string>())
const completedWidgets = ref(new Set<string>())
const resolvers: (() => void)[] = []

/**
 * Register a widget as pending creation.
 */
export function registerPendingWidget(id: string): void {
  pendingWidgets.value.add(id)
}

/**
 * Mark a widget as completed.
 */
export function markWidgetCompleted(id: string): void {
  pendingWidgets.value.delete(id)
  completedWidgets.value.add(id)

  // Check if all widgets are completed
  if (pendingWidgets.value.size === 0 && resolvers.length > 0) {
    resolvers.forEach(resolve => resolve())
    resolvers.length = 0
  }
}

/**
 * Wait for all pending widgets to complete.
 * Returns immediately if no widgets are pending.
 */
export function waitForAllWidgets(): Promise<void> {
  if (pendingWidgets.value.size === 0) {
    return Promise.resolve()
  }

  return new Promise<void>((resolve) => {
    resolvers.push(resolve)
  })
}

/**
 * Reset the registry (useful for testing or hot reload).
 */
export function resetWidgetRegistry(): void {
  pendingWidgets.value.clear()
  completedWidgets.value.clear()
  resolvers.length = 0
}

export const widgetRegistryState = {
  pending: readonly(pendingWidgets),
  completed: readonly(completedWidgets),
}
