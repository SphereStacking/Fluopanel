import { ref, computed, onUnmounted, type Ref, type ComputedRef } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  createPopoverController,
  getPopoverContext,
  isPopover as coreIsPopover,
  getPopoverId as coreGetPopoverId,
  type PopoverContext,
  type PopoverAnchor,
  type PopoverAlign,
} from '@arcana/core'

export type { PopoverAnchor, PopoverAlign }

/**
 * Vue composable for popover mode detection.
 * Use this to determine if the current window is a popover.
 */
export function usePopoverMode() {
  const context = ref<PopoverContext>(getPopoverContext())
  const isPopover = ref(coreIsPopover())
  const popoverId = ref<string | null>(coreGetPopoverId())

  return {
    context,
    isPopover,
    popoverId,
  }
}

export interface UsePopoverOptions {
  /** Popover width in pixels */
  width: number
  /** Popover height in pixels */
  height: number
  /** Alignment relative to anchor (default: 'end') */
  align?: PopoverAlign
  /** Vertical offset from anchor (default: 8) */
  offsetY?: number
  /**
   * Exclusive mode:
   * - true: close all other popovers
   * - string: close popovers with matching ID prefix (e.g., "github" closes "github-*")
   * - false/undefined: don't close anything
   */
  exclusive?: boolean | string
}

export interface UsePopoverReturn {
  /** Whether popover is currently open */
  isOpen: ComputedRef<boolean>
  /** Currently open popover ID */
  openPopoverId: Ref<string | null>
  /** Toggle popover open/closed */
  toggle: (triggerId: string, triggerElement: HTMLElement) => Promise<void>
  /** Close popover */
  close: () => Promise<void>
}

/**
 * Helper to get anchor coordinates from trigger element
 */
async function getAnchorFromElement(triggerElement: HTMLElement): Promise<PopoverAnchor> {
  const rect = triggerElement.getBoundingClientRect()
  const window = getCurrentWindow()
  const windowPos = await window.outerPosition()
  const scaleFactor = await window.scaleFactor()

  const windowX = windowPos.x / scaleFactor
  const windowY = windowPos.y / scaleFactor

  return {
    x: windowX + rect.x,
    y: windowY + rect.y,
    width: rect.width,
    height: rect.height,
  }
}

/**
 * Vue composable for managing popovers (Toggle mode).
 * Click to open/close, blur automatically closes.
 */
export function usePopover(options: UsePopoverOptions): UsePopoverReturn {
  const { width, height, align = 'end', offsetY = 8, exclusive = false } = options

  const controller = createPopoverController()
  const openPopoverId = ref<string | null>(null)
  const isOpen = computed(() => openPopoverId.value !== null)

  let popoverClosedUnlisten: UnlistenFn | null = null

  async function setupEventListeners() {
    if (!popoverClosedUnlisten) {
      console.log('[usePopover] Setting up listener')
      popoverClosedUnlisten = await listen<string>('popover-closed', (event) => {
        console.log('[usePopover] Received:', event.payload, 'current:', openPopoverId.value)
        if (event.payload === openPopoverId.value) {
          console.log('[usePopover] Closing popover')
          openPopoverId.value = null
        }
      })
    }
  }

  onUnmounted(() => {
    controller.destroy()
    if (popoverClosedUnlisten) {
      popoverClosedUnlisten()
      popoverClosedUnlisten = null
    }
  })

  async function toggle(triggerId: string, triggerElement: HTMLElement): Promise<void> {
    // リスナーを先にセットアップ（blur イベントを確実にキャッチ）
    await setupEventListeners()

    const anchor = await getAnchorFromElement(triggerElement)

    // Use toggle mode: open_popover returns { closed: true } if it closed an existing popover
    const result = await controller.open(
      {
        id: triggerId,
        anchor,
        width,
        height,
        align,
        offsetY,
      },
      exclusive
    )

    if (result.closed) {
      openPopoverId.value = null
    } else {
      openPopoverId.value = triggerId
    }
  }

  async function close(): Promise<void> {
    if (openPopoverId.value) {
      await controller.close(openPopoverId.value)
      openPopoverId.value = null
    }
  }

  return {
    isOpen,
    openPopoverId,
    toggle,
    close,
  }
}
