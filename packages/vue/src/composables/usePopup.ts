import { ref, computed, onMounted, onUnmounted, watch, type Ref, type ComputedRef } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  createPopupController,
  createTriggerManager,
  getPopupContext,
  isPopup as coreIsPopup,
  getPopupId as coreGetPopupId,
  type PopupContext,
  type PopupAnchor,
  type PopupAlign,
  type PopupMode,
  type TriggerManager,
  type TriggerBounds,
} from '@arcana/core'

export type { PopupAnchor, PopupAlign, PopupMode }

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
  /** Popup mode (default: 'toggle') */
  mode?: PopupMode
  /** Popup width in pixels */
  width: number
  /** Popup height in pixels */
  height: number
  /** Alignment relative to anchor (default: 'end') */
  align?: PopupAlign
  /** Vertical offset from anchor (default: 8) */
  offsetY?: number
  /**
   * For hover mode: trigger element ref to enable Rust-side global mouse monitoring.
   * When provided, hover triggers are registered with Rust and events are fired
   * even when the window is not focused.
   */
  triggerRef?: Ref<HTMLElement | null>
  /**
   * For hover mode: unique trigger ID (defaults to a generated ID)
   */
  triggerId?: string
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
  /**
   * Notify that mouse entered trigger element (for hover mode).
   * Call this on mouseenter of the trigger element.
   */
  onTriggerEnter: () => void
  /**
   * Notify that mouse left trigger element (for hover mode).
   * Call this on mouseleave of the trigger element.
   */
  onTriggerLeave: () => void
}

/**
 * Vue composable for managing popups.
 * For hover mode, hover state is coordinated between JS (trigger/popup events) and Rust (state management).
 * When triggerRef is provided for hover mode, Rust-side global mouse monitoring is used instead of JS events.
 */
export function usePopup(options: UsePopupOptions): UsePopupReturn {
  const { mode = 'toggle', width, height, align = 'end', offsetY = 8, triggerRef, triggerId: customTriggerId } = options

  const controller = createPopupController()
  const openPopupId = ref<string | null>(null)
  const isOpen = computed(() => openPopupId.value !== null)

  let popupClosedUnlisten: UnlistenFn | null = null
  let triggerManager: TriggerManager | null = null
  let triggerRegistered = false
  const generatedTriggerId = customTriggerId || `trigger-${Math.random().toString(36).slice(2, 9)}`

  /**
   * Convert element bounds from viewport to screen coordinates
   */
  async function getScreenBounds(el: HTMLElement): Promise<TriggerBounds> {
    const rect = el.getBoundingClientRect()
    const window = getCurrentWindow()
    const pos = await window.outerPosition()
    const scale = await window.scaleFactor()

    const windowX = pos.x / scale
    const windowY = pos.y / scale

    return {
      x: windowX + rect.x,
      y: windowY + rect.y,
      width: rect.width,
      height: rect.height,
    }
  }

  /**
   * Open popup using the triggerRef element for positioning
   */
  async function openFromTrigger(): Promise<void> {
    if (!triggerRef?.value) return

    const bounds = await getScreenBounds(triggerRef.value)
    const anchor: PopupAnchor = {
      x: bounds.x,
      y: bounds.y,
      width: bounds.width,
      height: bounds.height,
    }

    // Close existing popup if different
    if (openPopupId.value && openPopupId.value !== generatedTriggerId) {
      await controller.close(openPopupId.value)
    }

    await controller.open({
      id: generatedTriggerId,
      anchor,
      width,
      height,
      align,
      offsetY,
      mode,
    })

    openPopupId.value = generatedTriggerId
    await setupEventListeners()
  }

  // Setup event listeners
  async function setupEventListeners() {
    // Listen for popup-closed event from Rust (emitted when blur or hover closes the popup)
    if (!popupClosedUnlisten) {
      popupClosedUnlisten = await listen<string>('popup-closed', (event) => {
        if (event.payload === openPopupId.value) {
          openPopupId.value = null
        }
      })
    }
  }

  // Initialize trigger manager for hover mode with triggerRef
  onMounted(async () => {
    if (mode === 'hover' && triggerRef) {
      triggerManager = createTriggerManager()

      // When mouse enters trigger, open popup
      triggerManager.onHoverEnter((id) => {
        if (id === generatedTriggerId) {
          openFromTrigger().catch(console.error)
        }
      })

      // Hover leave is handled by Rust window number monitor
      // No action needed here - popup closes when mouse leaves both trigger and popup

      // Wait for element to be available
      if (triggerRef.value) {
        await registerTrigger()
      } else {
        // Watch for element to become available
        const stopWatch = watch(triggerRef, async (el) => {
          if (el && !triggerRegistered) {
            await registerTrigger()
            stopWatch()
          }
        })
      }
    }
  })

  async function registerTrigger() {
    if (!triggerRef?.value || !triggerManager || triggerRegistered) return

    try {
      const bounds = await getScreenBounds(triggerRef.value)
      await triggerManager.register({
        id: generatedTriggerId,
        bounds,
        popupWidth: width,
        popupHeight: height,
        popupAlign: align,
        popupOffsetY: offsetY,
      })
      triggerRegistered = true
    } catch (error) {
      console.error('[usePopup] Failed to register trigger:', error)
    }
  }

  onUnmounted(() => {
    controller.destroy()
    if (popupClosedUnlisten) {
      popupClosedUnlisten()
      popupClosedUnlisten = null
    }
    if (triggerManager) {
      if (triggerRegistered) {
        triggerManager.unregister(generatedTriggerId).catch(console.error)
      }
      triggerManager.destroy()
      triggerManager = null
      triggerRegistered = false
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
      mode,
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

  // Hover mode trigger tracking (legacy, for JS-based hover without triggerRef)
  function onTriggerEnter() {
    if (mode === 'hover' && openPopupId.value) {
      invoke('popup_trigger_enter', { popupId: openPopupId.value }).catch(console.error)
    }
  }

  function onTriggerLeave() {
    if (mode === 'hover' && openPopupId.value) {
      invoke('popup_trigger_leave', { popupId: openPopupId.value }).catch(console.error)
    }
  }

  return {
    isOpen,
    openPopupId,
    open,
    close,
    toggle,
    onTriggerEnter,
    onTriggerLeave,
  }
}
