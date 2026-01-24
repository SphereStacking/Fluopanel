import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { WindowPosition, MonitorInfo } from './types'

export interface WindowController {
  /** Get all available monitors */
  getMonitors(): Promise<MonitorInfo[]>

  /** Update position of a specific window */
  updateWindowPosition(label: string, position: WindowPosition): Promise<void>

  /** Subscribe to monitor changes */
  onMonitorChange(callback: () => void): Promise<UnlistenFn>
}

/**
 * Create a WindowController instance for managing window positioning.
 * Position calculation is done in Rust - TypeScript just passes the position config.
 */
export function createWindowController(): WindowController {
  return {
    async getMonitors(): Promise<MonitorInfo[]> {
      return invoke<MonitorInfo[]>('get_monitors')
    },

    async updateWindowPosition(label: string, position: WindowPosition): Promise<void> {
      await invoke('update_window_position', {
        label,
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
    },

    onMonitorChange(callback: () => void): Promise<UnlistenFn> {
      return listen('monitor-changed', callback)
    },
  }
}
