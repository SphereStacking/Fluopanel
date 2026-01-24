import { invoke } from '@tauri-apps/api/core'
import type {
  InlineWindowOptions,
  WindowContext,
  WindowPosition,
} from './types'
import { createWindowController } from './window-controller'

const windowController = createWindowController()

/**
 * Get the current window context from URL parameters.
 * Returns 'coordinator' mode if no window or float parameter is present.
 */
export function getWindowContext(): WindowContext {
  const params = new URLSearchParams(window.location.search)
  const windowId = params.get('window')
  const floatId = params.get('float')

  if (windowId) {
    return {
      id: windowId,
      label: `inline-window-${windowId}`,
      mode: 'window',
    }
  }

  // Float windows are also not coordinators
  if (floatId) {
    return {
      id: floatId,
      label: `float-${floatId}`,
      mode: 'window', // Treat float as window (not coordinator)
    }
  }

  return {
    id: 'coordinator',
    label: 'main',
    mode: 'coordinator',
  }
}

/**
 * Check if current window is running as coordinator (main entry point)
 */
export function isCoordinator(): boolean {
  return getWindowContext().mode === 'coordinator'
}

/**
 * Check if current window is running as a window
 */
export function isWindow(): boolean {
  return getWindowContext().mode === 'window'
}

/**
 * Get the window ID if running as a window, or null if coordinator
 */
export function getWindowId(): string | null {
  const ctx = getWindowContext()
  return ctx.mode === 'window' ? ctx.id : null
}

/**
 * Create a new inline window.
 * Position calculation is done in Rust.
 * Should only be called from coordinator mode.
 */
export async function createInlineWindow(
  options: InlineWindowOptions
): Promise<void> {
  const { id, position, window: windowConfig, url } = options

  // Build the URL for the window
  // By default, use the current origin with ?window=id parameter
  const windowUrl = url ?? `${window.location.origin}${window.location.pathname}?window=${id}`

  // Create window - Rust handles position calculation
  await invoke('create_inline_window', {
    windowId: id,
    url: windowUrl,
    transparent: windowConfig?.transparent ?? true,
    alwaysOnTop: windowConfig?.alwaysOnTop ?? true,
    decorations: windowConfig?.decorations ?? false,
    resizable: windowConfig?.resizable ?? false,
    skipTaskbar: windowConfig?.skipTaskbar ?? true,
    position: {
      monitor: position.monitor,
      top: position.top,
      bottom: position.bottom,
      left: position.left,
      right: position.right,
      width: position.width,
      height: position.height,
    },
  })

  // Show the window
  const label = `inline-window-${id}`
  await invoke('show_window', { label })
}

/**
 * Close an inline window
 */
export async function closeInlineWindow(id: string): Promise<void> {
  const label = `inline-window-${id}`
  await invoke('close_window', { label })
}

/**
 * Update the position of an existing inline window
 */
export async function updateWindowPosition(
  id: string,
  position: WindowPosition
): Promise<void> {
  const label = `inline-window-${id}`
  await windowController.updateWindowPosition(label, position)
}

/**
 * Hide the coordinator window (make it invisible).
 * Useful when all UI is rendered in windows.
 */
export async function hideCoordinatorWindow(): Promise<void> {
  await invoke('hide_window', { label: 'main' })
}

/**
 * Window manager for frameworks to use.
 * Provides the core functionality without framework-specific bindings.
 */
export interface WindowManager {
  context: WindowContext
  isCoordinator: boolean
  isWindow: boolean
  windowId: string | null
  create: (options: InlineWindowOptions) => Promise<void>
  close: (id: string) => Promise<void>
  updatePosition: (id: string, position: WindowPosition) => Promise<void>
  hideCoordinator: () => Promise<void>
}

/**
 * Create a window manager instance.
 * Framework-agnostic - can be wrapped by Vue/React/Solid composables/hooks.
 */
export function createWindowManager(): WindowManager {
  const context = getWindowContext()

  return {
    context,
    isCoordinator: context.mode === 'coordinator',
    isWindow: context.mode === 'window',
    windowId: context.mode === 'window' ? context.id : null,
    create: createInlineWindow,
    close: closeInlineWindow,
    updatePosition: updateWindowPosition,
    hideCoordinator: hideCoordinatorWindow,
  }
}
