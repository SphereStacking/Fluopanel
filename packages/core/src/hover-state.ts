/**
 * HoverStateManager - Unified hover region management
 * Framework-independent pure TypeScript implementation
 */

export interface HoverStateManagerOptions {
  /** Delay in ms before closing when mouse leaves both trigger and popup */
  leaveDelay: number
  /** Callback to close the popup */
  onClose: () => void
}

export interface HoverStateManager {
  /** Mouse entered the trigger element */
  triggerEnter(): void
  /** Mouse left the trigger element */
  triggerLeave(): void
  /** Mouse entered the popup window */
  popupEnter(): void
  /** Mouse left the popup window */
  popupLeave(): void
  /** Force close (for cleanup) */
  forceClose(): void
  /** Release resources */
  destroy(): void
}

/**
 * Creates a hover state manager that tracks mouse presence
 * across both trigger element and popup window.
 *
 * Closes the popup only when mouse has left BOTH areas
 * for the specified delay duration.
 */
export function createHoverStateManager(
  options: HoverStateManagerOptions
): HoverStateManager {
  const { leaveDelay, onClose } = options

  let isOverTrigger = false
  let isOverPopup = false
  let closeTimer: ReturnType<typeof setTimeout> | null = null
  let isDestroyed = false

  function clearTimer() {
    if (closeTimer) {
      clearTimeout(closeTimer)
      closeTimer = null
    }
  }

  function scheduleClose() {
    clearTimer()
    closeTimer = setTimeout(() => {
      if (!isDestroyed && !isOverTrigger && !isOverPopup) {
        onClose()
      }
    }, leaveDelay)
  }

  function checkState() {
    if (isOverTrigger || isOverPopup) {
      // Mouse is over either area - cancel any pending close
      clearTimer()
    } else {
      // Mouse has left both areas - schedule close after delay
      scheduleClose()
    }
  }

  return {
    triggerEnter() {
      if (isDestroyed) return
      isOverTrigger = true
      checkState()
    },

    triggerLeave() {
      if (isDestroyed) return
      isOverTrigger = false
      checkState()
    },

    popupEnter() {
      if (isDestroyed) return
      isOverPopup = true
      checkState()
    },

    popupLeave() {
      if (isDestroyed) return
      isOverPopup = false
      checkState()
    },

    forceClose() {
      clearTimer()
      if (!isDestroyed) {
        onClose()
      }
    },

    destroy() {
      isDestroyed = true
      clearTimer()
    },
  }
}
