import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { PopupOpenOptions, PopupInfo, PopupContext, PopupAnchor, PopupAlign } from './types'

/**
 * Get current popup context from URL parameters
 */
export function getPopupContext(): PopupContext {
  if (typeof window === 'undefined') {
    return { id: null, isPopup: false }
  }

  const params = new URLSearchParams(window.location.search)
  const popupId = params.get('popup')

  return {
    id: popupId,
    isPopup: popupId !== null,
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
  }
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
 * Listen for popup closed events
 * Returns unsubscribe function
 */
export async function onPopupClosed(
  callback: (popupId: string) => void
): Promise<UnlistenFn> {
  return await listen<string>('popup-closed', (event) => {
    callback(event.payload)
  })
}

/**
 * Create a popup controller instance
 * Manages popup lifecycle and close events
 */
export function createPopupController() {
  let closeUnlisten: UnlistenFn | null = null
  const closeCallbacks = new Map<string, () => void>()

  const ensureCloseListener = async () => {
    if (closeUnlisten) return

    closeUnlisten = await onPopupClosed((popupId) => {
      const callback = closeCallbacks.get(popupId)
      if (callback) {
        callback()
      }
    })
  }

  return {
    /**
     * Open a popup and optionally register close callback
     */
    async open(
      options: PopupOpenOptions,
      onClose?: () => void
    ): Promise<PopupInfo> {
      if (onClose) {
        await ensureCloseListener()
        closeCallbacks.set(options.id, onClose)
      }

      return await openPopup(options)
    },

    /**
     * Close a popup
     */
    async close(popupId: string): Promise<void> {
      closeCallbacks.delete(popupId)
      await closePopup(popupId)
    },

    /**
     * Close all popups
     */
    async closeAll(): Promise<void> {
      closeCallbacks.clear()
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
      if (closeUnlisten) {
        closeUnlisten()
        closeUnlisten = null
      }
      closeCallbacks.clear()
    },
  }
}

export type PopupController = ReturnType<typeof createPopupController>
