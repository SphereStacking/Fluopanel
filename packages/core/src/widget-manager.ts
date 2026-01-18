import { invoke } from '@tauri-apps/api/core'
import type { WidgetManifest, WidgetInstance } from './types'

export interface WidgetManager {
  /** Discover all widgets in the widgets directory */
  discoverWidgets(): Promise<WidgetManifest[]>

  /** Get a specific widget manifest by ID */
  getWidgetManifest(widgetId: string): Promise<WidgetManifest>

  /** Create a new widget window */
  createWidget(instance: WidgetInstance): Promise<void>

  /** Close a widget window */
  closeWidget(instanceId: string): Promise<void>

  /** Get all active widget window labels */
  getActiveWidgets(): Promise<string[]>

  /** Show a widget window (after positioning) */
  showWidget(label: string): Promise<void>
}

/**
 * Create a WidgetManager instance for managing widgets.
 * This is framework-agnostic and can be used with any frontend framework.
 */
export function createWidgetManager(): WidgetManager {
  return {
    async discoverWidgets(): Promise<WidgetManifest[]> {
      return invoke<WidgetManifest[]>('discover_widgets')
    },

    async getWidgetManifest(widgetId: string): Promise<WidgetManifest> {
      return invoke<WidgetManifest>('get_widget_manifest', { widgetId })
    },

    async createWidget(instance: WidgetInstance): Promise<void> {
      const manifest = await this.getWidgetManifest(instance.widgetId)
      return invoke('create_widget_window', {
        widgetId: instance.widgetId,
        instanceId: instance.instanceId,
        manifest,
      })
    },

    async closeWidget(instanceId: string): Promise<void> {
      // Find the label from the instanceId
      const windows = await this.getActiveWidgets()
      const label = windows.find(w => w.includes(instanceId))
      if (label) {
        return invoke('close_widget_window', { label })
      }
    },

    async getActiveWidgets(): Promise<string[]> {
      return invoke<string[]>('get_widget_windows')
    },

    async showWidget(label: string): Promise<void> {
      return invoke('show_widget_window', { label })
    },
  }
}
