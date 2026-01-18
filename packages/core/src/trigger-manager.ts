import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { PopupAlign } from './types'

/**
 * Trigger bounds in screen coordinates (JS top-left origin)
 */
export interface TriggerBounds {
  x: number
  y: number
  width: number
  height: number
}

/**
 * Options for registering a hover trigger
 */
export interface TriggerOptions {
  /** Unique trigger ID */
  id: string
  /** Trigger bounds in screen coordinates */
  bounds: TriggerBounds
  /** Popup width in pixels */
  popupWidth: number
  /** Popup height in pixels */
  popupHeight: number
  /** Popup alignment (default: 'center') */
  popupAlign?: PopupAlign
  /** Popup vertical offset (default: 8) */
  popupOffsetY?: number
}

/**
 * Callback type for hover events
 */
export type TriggerHoverCallback = (triggerId: string) => void

/**
 * TriggerManager interface - manages trigger registration and hover events
 * This is framework-agnostic and can be used with Vue, React, etc.
 */
export interface TriggerManager {
  /**
   * Register a hover trigger for global mouse monitoring
   */
  register(options: TriggerOptions): Promise<void>

  /**
   * Unregister a hover trigger
   */
  unregister(triggerId: string): Promise<void>

  /**
   * Update trigger bounds (e.g., after window resize/move)
   */
  updateBounds(triggerId: string, bounds: TriggerBounds): Promise<void>

  /**
   * Subscribe to hover enter events
   */
  onHoverEnter(callback: TriggerHoverCallback): void

  /**
   * Subscribe to hover leave events
   */
  onHoverLeave(callback: TriggerHoverCallback): void

  /**
   * Cleanup resources (event listeners)
   */
  destroy(): void
}

/**
 * Create a TriggerManager instance
 * This manages trigger registration with Rust and listens for hover events
 */
export function createTriggerManager(): TriggerManager {
  let enterUnlisten: UnlistenFn | null = null
  let leaveUnlisten: UnlistenFn | null = null
  const enterCallbacks: TriggerHoverCallback[] = []
  const leaveCallbacks: TriggerHoverCallback[] = []
  let listenersInitialized = false

  async function ensureListeners() {
    if (listenersInitialized) return

    listenersInitialized = true

    enterUnlisten = await listen<string>('trigger-hover-enter', (event) => {
      enterCallbacks.forEach((cb) => cb(event.payload))
    })

    leaveUnlisten = await listen<string>('trigger-hover-leave', (event) => {
      leaveCallbacks.forEach((cb) => cb(event.payload))
    })
  }

  return {
    async register(options) {
      // Ensure event listeners are set up
      await ensureListeners()

      await invoke('register_hover_trigger', {
        triggerId: options.id,
        bounds: options.bounds,
        popupWidth: options.popupWidth,
        popupHeight: options.popupHeight,
        popupAlign: options.popupAlign ?? 'center',
        popupOffsetY: options.popupOffsetY ?? 8,
      })
    },

    async unregister(triggerId) {
      await invoke('unregister_hover_trigger', { triggerId })
    },

    async updateBounds(triggerId, bounds) {
      await invoke('update_trigger_bounds', { triggerId, bounds })
    },

    onHoverEnter(callback) {
      enterCallbacks.push(callback)
    },

    onHoverLeave(callback) {
      leaveCallbacks.push(callback)
    },

    destroy() {
      enterUnlisten?.()
      leaveUnlisten?.()
      enterUnlisten = null
      leaveUnlisten = null
      listenersInitialized = false
      enterCallbacks.length = 0
      leaveCallbacks.length = 0
    },
  }
}

export type { PopupAlign }
