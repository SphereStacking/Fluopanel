import { invoke } from '@tauri-apps/api/core'
import type { WindowManifest, WindowInstance } from './types'

export interface WindowDiscovery {
  /** Discover all windows in the windows directory */
  discoverWindows(): Promise<WindowManifest[]>

  /** Get a specific window manifest by ID */
  getWindowManifest(windowId: string): Promise<WindowManifest>

  /** Create a new window */
  createWindow(instance: WindowInstance): Promise<void>

  /** Close a window */
  closeWindow(instanceId: string): Promise<void>

  /** Get all active window labels */
  getActiveWindows(): Promise<string[]>

  /** Show a window (after positioning) */
  showWindow(label: string): Promise<void>
}

/**
 * Create a WindowDiscovery instance for managing windows.
 * This is framework-agnostic and can be used with any frontend framework.
 */
export function createWindowDiscovery(): WindowDiscovery {
  return {
    async discoverWindows(): Promise<WindowManifest[]> {
      return invoke<WindowManifest[]>('discover_windows')
    },

    async getWindowManifest(windowId: string): Promise<WindowManifest> {
      return invoke<WindowManifest>('get_window_manifest', { windowId })
    },

    async createWindow(instance: WindowInstance): Promise<void> {
      const manifest = await this.getWindowManifest(instance.windowId)
      return invoke('create_window', {
        windowId: instance.windowId,
        instanceId: instance.instanceId,
        manifest,
      })
    },

    async closeWindow(instanceId: string): Promise<void> {
      // Find the label from the instanceId
      const windows = await this.getActiveWindows()
      const label = windows.find(w => w.includes(instanceId))
      if (label) {
        return invoke('close_window', { label })
      }
    },

    async getActiveWindows(): Promise<string[]> {
      return invoke<string[]>('get_windows')
    },

    async showWindow(label: string): Promise<void> {
      return invoke('show_window', { label })
    },
  }
}
