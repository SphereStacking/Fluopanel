import { invoke } from '@tauri-apps/api/core'
import type {
  InlineWidgetOptions,
  WidgetContext,
  WidgetPosition,
} from './types'
import { createWindowController } from './window-controller'

const windowController = createWindowController()

/**
 * Get the current widget context from URL parameters.
 * Returns 'coordinator' mode if no widget parameter is present.
 */
export function getWidgetContext(): WidgetContext {
  const params = new URLSearchParams(window.location.search)
  const widgetId = params.get('widget')

  if (widgetId) {
    return {
      id: widgetId,
      label: `inline-widget-${widgetId}`,
      mode: 'widget',
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
  return getWidgetContext().mode === 'coordinator'
}

/**
 * Check if current window is running as a widget
 */
export function isWidget(): boolean {
  return getWidgetContext().mode === 'widget'
}

/**
 * Get the widget ID if running as a widget, or null if coordinator
 */
export function getWidgetId(): string | null {
  const ctx = getWidgetContext()
  return ctx.mode === 'widget' ? ctx.id : null
}

/**
 * Create a new inline widget window.
 * Position calculation is done in Rust.
 * Should only be called from coordinator mode.
 */
export async function createInlineWidgetWindow(
  options: InlineWidgetOptions
): Promise<void> {
  const { id, position, window: windowConfig, url } = options

  // Build the URL for the widget window
  // By default, use the current origin with ?widget=id parameter
  const widgetUrl = url ?? `${window.location.origin}${window.location.pathname}?widget=${id}`

  // Create window - Rust handles position calculation
  await invoke('create_inline_widget_window', {
    widgetId: id,
    url: widgetUrl,
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
  const label = `inline-widget-${id}`
  await invoke('show_widget_window', { label })
}

/**
 * Close an inline widget window
 */
export async function closeInlineWidgetWindow(id: string): Promise<void> {
  const label = `inline-widget-${id}`
  await invoke('close_widget_window', { label })
}

/**
 * Update the position of an existing inline widget window
 */
export async function updateWidgetPosition(
  id: string,
  position: WidgetPosition
): Promise<void> {
  const label = `inline-widget-${id}`
  await windowController.updateWindowPosition(label, position)
}

/**
 * Hide the coordinator window (make it invisible).
 * Useful when all UI is rendered in widget windows.
 */
export async function hideCoordinatorWindow(): Promise<void> {
  await invoke('hide_window', { label: 'main' })
}

/**
 * Widget window manager for frameworks to use.
 * Provides the core functionality without framework-specific bindings.
 */
export interface WidgetWindowManager {
  context: WidgetContext
  isCoordinator: boolean
  isWidget: boolean
  widgetId: string | null
  create: (options: InlineWidgetOptions) => Promise<void>
  close: (id: string) => Promise<void>
  updatePosition: (id: string, position: WidgetPosition) => Promise<void>
  hideCoordinator: () => Promise<void>
}

/**
 * Create a widget window manager instance.
 * Framework-agnostic - can be wrapped by Vue/React/Solid composables/hooks.
 */
export function createWidgetWindowManager(): WidgetWindowManager {
  const context = getWidgetContext()

  return {
    context,
    isCoordinator: context.mode === 'coordinator',
    isWidget: context.mode === 'widget',
    widgetId: context.mode === 'widget' ? context.id : null,
    create: createInlineWidgetWindow,
    close: closeInlineWidgetWindow,
    updatePosition: updateWidgetPosition,
    hideCoordinator: hideCoordinatorWindow,
  }
}
