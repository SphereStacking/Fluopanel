import { ref, onMounted, onUnmounted, type Ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  createTriggerManager,
  type TriggerBounds,
  type TriggerManager,
  type PopupAlign,
} from '@arcana/core'

export type { TriggerBounds, PopupAlign }

/**
 * Options for useTrigger composable
 */
export interface UseTriggerOptions {
  /** Unique trigger ID */
  triggerId: string
  /** Ref to the trigger element */
  elementRef: Ref<HTMLElement | null>
  /** Popup width in pixels */
  popupWidth: number
  /** Popup height in pixels */
  popupHeight: number
  /** Popup alignment (default: 'center') */
  popupAlign?: PopupAlign
  /** Popup vertical offset (default: 8) */
  popupOffsetY?: number
  /** Callback when mouse enters trigger */
  onHoverEnter?: () => void
  /** Callback when mouse leaves trigger */
  onHoverLeave?: () => void
}

/**
 * Return type for useTrigger composable
 */
export interface UseTriggerReturn {
  /** Whether mouse is currently over the trigger */
  isHovering: Ref<boolean>
  /** Manually update trigger bounds (e.g., after resize) */
  updateBounds: () => Promise<void>
}

/**
 * Vue composable for managing hover triggers with Rust-side global mouse monitoring.
 * This allows hover detection even when the window is not focused.
 *
 * @example
 * ```vue
 * <script setup>
 * const triggerRef = ref<HTMLElement | null>(null)
 * const { isHovering } = useTrigger({
 *   triggerId: 'my-trigger',
 *   elementRef: triggerRef,
 *   popupWidth: 300,
 *   popupHeight: 400,
 *   onHoverEnter: () => openPopup(),
 *   onHoverLeave: () => closePopup(),
 * })
 * </script>
 *
 * <template>
 *   <div ref="triggerRef">Hover me</div>
 * </template>
 * ```
 */
export function useTrigger(options: UseTriggerOptions): UseTriggerReturn {
  const {
    triggerId,
    elementRef,
    popupWidth,
    popupHeight,
    popupAlign = 'center',
    popupOffsetY = 8,
    onHoverEnter,
    onHoverLeave,
  } = options

  const isHovering = ref(false)
  let manager: TriggerManager | null = null
  let registered = false

  /**
   * Convert element bounds from viewport to screen coordinates
   */
  async function getScreenBounds(el: HTMLElement): Promise<TriggerBounds> {
    const rect = el.getBoundingClientRect()
    const window = getCurrentWindow()
    const pos = await window.outerPosition()
    const scale = await window.scaleFactor()

    // Convert window position from physical to logical pixels
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
   * Update trigger bounds with Rust
   */
  async function updateBounds(): Promise<void> {
    if (!elementRef.value || !manager || !registered) return

    try {
      const bounds = await getScreenBounds(elementRef.value)
      await manager.updateBounds(triggerId, bounds)
    } catch (error) {
      console.error('[useTrigger] Failed to update bounds:', error)
    }
  }

  onMounted(async () => {
    if (!elementRef.value) return

    manager = createTriggerManager()

    // Set up hover callbacks
    manager.onHoverEnter((id) => {
      if (id === triggerId) {
        isHovering.value = true
        onHoverEnter?.()
      }
    })

    manager.onHoverLeave((id) => {
      if (id === triggerId) {
        isHovering.value = false
        onHoverLeave?.()
      }
    })

    // Register trigger with Rust
    try {
      const bounds = await getScreenBounds(elementRef.value)
      await manager.register({
        id: triggerId,
        bounds,
        popupWidth,
        popupHeight,
        popupAlign,
        popupOffsetY,
      })
      registered = true
    } catch (error) {
      console.error('[useTrigger] Failed to register trigger:', error)
    }
  })

  onUnmounted(() => {
    if (manager) {
      if (registered) {
        manager.unregister(triggerId).catch(console.error)
      }
      manager.destroy()
      manager = null
      registered = false
    }
  })

  return {
    isHovering,
    updateBounds,
  }
}
