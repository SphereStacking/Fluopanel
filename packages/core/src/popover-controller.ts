import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type {
  PopoverOpenOptions,
  PopoverInfo,
  PopoverContext,
  PopoverAnchor,
  PopoverAlign,
} from './types'

/**
 * Get current popover context from URL parameters
 */
export function getPopoverContext(): PopoverContext {
  if (typeof window === 'undefined') {
    return { id: null, isPopover: false }
  }

  const params = new URLSearchParams(window.location.search)
  const popoverId = params.get('popover')

  return {
    id: popoverId,
    isPopover: popoverId !== null,
  }
}

/**
 * Check if current window is a popover
 */
export function isPopover(): boolean {
  return getPopoverContext().isPopover
}

/**
 * Get current popover ID (null if not a popover)
 */
export function getPopoverId(): string | null {
  return getPopoverContext().id
}

/**
 * Open a popover window below the anchor element (toggle mode)
 * If popover already exists, it will be closed.
 */
export async function openPopover(options: PopoverOpenOptions): Promise<PopoverInfo> {
  const params = {
    popoverId: options.id,
    anchor: options.anchor,
    width: options.width,
    height: options.height,
    align: options.align ?? 'center',
    offsetY: options.offsetY ?? 8,
  }
  return await invoke<PopoverInfo>('open_popover', params)
}

/**
 * Close a popover window by ID
 */
export async function closePopover(popoverId: string): Promise<void> {
  await invoke('close_popover', { popoverId })
}

/**
 * Close all open popover windows
 */
export async function closeAllPopovers(): Promise<void> {
  await invoke('close_all_popovers')
}

/**
 * Get list of open popover IDs
 */
export async function getOpenPopovers(): Promise<string[]> {
  return await invoke<string[]>('get_open_popovers')
}

/**
 * Listen for popover closed events
 * Returns unsubscribe function
 */
export async function onPopoverClosed(
  callback: (popoverId: string) => void
): Promise<UnlistenFn> {
  return await listen<string>('popover-closed', (event) => {
    callback(event.payload)
  })
}

/**
 * Create a popover controller instance
 * Manages popover lifecycle and close events
 */
export function createPopoverController() {
  let closeUnlisten: UnlistenFn | null = null
  const closeCallbacks = new Map<string, () => void>()

  const ensureCloseListener = async () => {
    if (closeUnlisten) return

    closeUnlisten = await onPopoverClosed((popoverId) => {
      const callback = closeCallbacks.get(popoverId)
      if (callback) {
        callback()
        closeCallbacks.delete(popoverId)
      }
    })
  }

  return {
    /**
     * Open a popover (toggle mode: opens if closed, closes if open)
     * @param options - Popover options
     * @param exclusive - Exclusive mode:
     *   - true: close all other popovers before opening
     *   - string: close popovers with matching ID prefix (e.g., "github" closes "github-*")
     *   - false/undefined: don't close anything
     * @param onClose - Callback when popover is closed
     */
    async open(
      options: PopoverOpenOptions,
      exclusive?: boolean | string,
      onClose?: () => void
    ): Promise<PopoverInfo> {
      if (exclusive) {
        const openPopovers = await getOpenPopovers()

        let popoversToClose: string[]
        if (exclusive === true) {
          // Close all except self
          popoversToClose = openPopovers.filter(id => id !== options.id)
        } else {
          // Close only matching group (ID prefix)
          const prefix = `${exclusive}-`
          popoversToClose = openPopovers.filter(
            id => id !== options.id && id.startsWith(prefix)
          )
        }

        // Close matching popovers
        for (const id of popoversToClose) {
          closeCallbacks.delete(id)
          await closePopover(id)
        }
      }

      if (onClose) {
        await ensureCloseListener()
        closeCallbacks.set(options.id, onClose)
      }

      return await openPopover(options)
    },

    /**
     * Close a popover
     */
    async close(popoverId: string): Promise<void> {
      closeCallbacks.delete(popoverId)
      await closePopover(popoverId)
    },

    /**
     * Close all popovers
     */
    async closeAll(): Promise<void> {
      closeCallbacks.clear()
      await closeAllPopovers()
    },

    /**
     * Get open popover IDs
     */
    getOpen: getOpenPopovers,

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

export type PopoverController = ReturnType<typeof createPopoverController>
