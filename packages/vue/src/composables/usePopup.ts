import { ref, computed, onUnmounted, type Ref, type ComputedRef } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  createPopupController,
  getPopupContext,
  isPopup as coreIsPopup,
  getPopupId as coreGetPopupId,
  type PopupContext,
  type PopupAnchor,
  type PopupAlign,
} from '@arcana/core'

export type { PopupAnchor, PopupAlign }

/**
 * Vue composable for popup mode detection.
 * Use this to determine if the current window is a popup.
 */
export function usePopupMode() {
  const context = ref<PopupContext>(getPopupContext())
  const isPopup = ref(coreIsPopup())
  const popupId = ref<string | null>(coreGetPopupId())

  return {
    context,
    isPopup,
    popupId,
  }
}

export interface UsePopupOptions {
  /** Popup width in pixels */
  width: number
  /** Popup height in pixels */
  height: number
  /** Alignment relative to anchor (default: 'end') */
  align?: PopupAlign
  /** Vertical offset from anchor (default: 8) */
  offsetY?: number
}

export interface UsePopupReturn {
  /** Whether popup is currently open */
  isOpen: ComputedRef<boolean>
  /** Currently open popup ID */
  openPopupId: Ref<string | null>
  /** Open popup below the trigger element */
  open: (triggerId: string, triggerElement: HTMLElement) => Promise<void>
  /** Close popup */
  close: () => Promise<void>
  /** Toggle popup open/closed */
  toggle: (triggerId: string, triggerElement: HTMLElement) => Promise<void>
}

/**
 * Vue composable for managing popups (Toggle mode).
 * Click to open, blur to close.
 */
export function usePopup(options: UsePopupOptions): UsePopupReturn {
  const { width, height, align = 'end', offsetY = 8 } = options

  const controller = createPopupController()
  const openPopupId = ref<string | null>(null)
  const isOpen = computed(() => openPopupId.value !== null)

  let popupClosedUnlisten: UnlistenFn | null = null

  // Setup event listeners
  async function setupEventListeners() {
    if (!popupClosedUnlisten) {
      popupClosedUnlisten = await listen<string>('popup-closed', (event) => {
        if (event.payload === openPopupId.value) {
          openPopupId.value = null
        }
      })
    }
  }

  onUnmounted(() => {
    controller.destroy()
    if (popupClosedUnlisten) {
      popupClosedUnlisten()
      popupClosedUnlisten = null
    }
  })

  async function open(triggerId: string, triggerElement: HTMLElement): Promise<void> {
    // Close existing popup if different
    if (openPopupId.value && openPopupId.value !== triggerId) {
      await controller.close(openPopupId.value)
    }

    // Get trigger element position (viewport coordinates)
    const rect = triggerElement.getBoundingClientRect()

    // Get parent window position to convert to screen coordinates
    const window = getCurrentWindow()
    const windowPos = await window.outerPosition()
    const scaleFactor = await window.scaleFactor()

    // Convert window position from physical to logical pixels
    const windowX = windowPos.x / scaleFactor
    const windowY = windowPos.y / scaleFactor

    // Create anchor with screen coordinates
    const anchor: PopupAnchor = {
      x: windowX + rect.x,
      y: windowY + rect.y,
      width: rect.width,
      height: rect.height,
    }

    await controller.open({
      id: triggerId,
      anchor,
      width,
      height,
      align,
      offsetY,
    })

    openPopupId.value = triggerId

    // Setup event listeners after opening
    await setupEventListeners()
  }

  async function close(): Promise<void> {
    if (openPopupId.value) {
      await controller.close(openPopupId.value)
      openPopupId.value = null
    }
  }

  async function toggle(triggerId: string, triggerElement: HTMLElement): Promise<void> {
    if (openPopupId.value === triggerId) {
      await close()
    } else {
      await open(triggerId, triggerElement)
    }
  }

  return {
    isOpen,
    openPopupId,
    open,
    close,
    toggle,
  }
}
