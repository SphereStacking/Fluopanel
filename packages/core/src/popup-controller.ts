import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { PopupOpenOptions, PopupInfo, PopupContext, PopupAnchor, PopupAlign, PopupMode } from './types'

/**
 * Get current popup context from URL parameters
 */
export function getPopupContext(): PopupContext {
  if (typeof window === 'undefined') {
    return { id: null, isPopup: false, mode: 'toggle' }
  }

  const params = new URLSearchParams(window.location.search)
  const popupId = params.get('popup')
  const modeParam = params.get('mode')
  const mode: PopupMode = (modeParam === 'hover' || modeParam === 'hover-sticky') ? modeParam : 'toggle'

  return {
    id: popupId,
    isPopup: popupId !== null,
    mode,
  }
}

/**
 * Check if current window is a popup
 */
export function isPopup(): boolean {
  return getPopupContext().isPopup
}

/**
 * Get current popup ID (null if not a popup)
 */
export function getPopupId(): string | null {
  return getPopupContext().id
}

/**
 * Get current popup mode
 */
export function getPopupMode(): PopupMode {
  return getPopupContext().mode
}

/**
 * Open a popup window below the anchor element
 */
export async function openPopup(options: PopupOpenOptions): Promise<PopupInfo> {
  const params = {
    popupId: options.id,
    anchor: options.anchor,
    width: options.width,
    height: options.height,
    align: options.align ?? 'center',
    offsetY: options.offsetY ?? 8,
    mode: options.mode ?? 'toggle',
  }
  console.log('[openPopup] Invoking create_popup_window with:', params)
  const result = await invoke<PopupInfo>('create_popup_window', params)

  return result
}

/**
 * Close a popup window by ID
 */
export async function closePopup(popupId: string): Promise<void> {
  await invoke('close_popup_window', { popupId })
}

/**
 * Close all open popup windows
 */
export async function closeAllPopups(): Promise<void> {
  await invoke('close_all_popups')
}

/**
 * Get list of open popup IDs
 */
export async function getOpenPopups(): Promise<string[]> {
  return await invoke<string[]>('get_open_popups')
}

/**
 * Update popup position (for repositioning when anchor moves)
 */
export async function updatePopupPosition(
  popupId: string,
  anchor: PopupAnchor,
  width: number,
  height: number,
  align?: PopupAlign,
  offsetY?: number
): Promise<void> {
  await invoke('update_popup_position', {
    popupId,
    anchor,
    width,
    height,
    align: align ?? 'center',
    offsetY: offsetY ?? 8,
  })
}

/**
 * Listen for popup blur events (focus lost)
 * Returns unsubscribe function
 */
export async function onPopupBlur(
  callback: (popupId: string) => void
): Promise<UnlistenFn> {
  return await listen<string>('popup-blur', (event) => {
    callback(event.payload)
  })
}

/**
 * Create a popup controller instance
 * Manages popup lifecycle and blur events
 * Note: Hover state management is handled by Rust side
 */
export function createPopupController() {
  let blurUnlisten: UnlistenFn | null = null
  const blurCallbacks = new Map<string, () => void>()

  const ensureBlurListener = async () => {
    if (blurUnlisten) return

    blurUnlisten = await onPopupBlur((popupId) => {
      const callback = blurCallbacks.get(popupId)
      if (callback) {
        callback()
      }
    })
  }

  return {
    /**
     * Open a popup and optionally register blur callback
     */
    async open(
      options: PopupOpenOptions,
      onBlur?: () => void
    ): Promise<PopupInfo> {
      // Ensure blur listener is active
      if (onBlur) {
        await ensureBlurListener()
        blurCallbacks.set(options.id, onBlur)
      }

      return await openPopup(options)
    },

    /**
     * Close a popup
     */
    async close(popupId: string): Promise<void> {
      blurCallbacks.delete(popupId)
      await closePopup(popupId)
    },

    /**
     * Close all popups
     */
    async closeAll(): Promise<void> {
      blurCallbacks.clear()
      await closeAllPopups()
    },

    /**
     * Get open popup IDs
     */
    getOpen: getOpenPopups,

    /**
     * Update popup position
     */
    updatePosition: updatePopupPosition,

    /**
     * Cleanup resources
     */
    destroy() {
      if (blurUnlisten) {
        blurUnlisten()
        blurUnlisten = null
      }
      blurCallbacks.clear()
    },
  }
}

export type PopupController = ReturnType<typeof createPopupController>
