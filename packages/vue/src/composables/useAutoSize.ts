import { ref, onMounted, onUnmounted, nextTick, type Ref, type ComputedRef } from 'vue'
import { invoke } from '@tauri-apps/api/core'

/** Value that can be a number, Ref<number>, or ComputedRef<number> */
type MaybeRef<T> = T | Ref<T> | ComputedRef<T>

export interface UseAutoSizeOptions {
  /** Whether auto-sizing is enabled (default: true) */
  enabled?: Ref<boolean> | boolean
  /** Minimum width in pixels */
  minWidth?: MaybeRef<number>
  /** Minimum height in pixels */
  minHeight?: MaybeRef<number>
  /** Maximum width in pixels */
  maxWidth?: MaybeRef<number>
  /** Maximum height in pixels */
  maxHeight?: MaybeRef<number>
}

export interface UseAutoSizeReturn {
  /** Current measured width */
  width: Ref<number>
  /** Current measured height */
  height: Ref<number>
  /** Manually trigger size update */
  updateSize: () => Promise<void>
}

/**
 * Vue composable for auto-sizing windows based on content.
 * Uses ResizeObserver to monitor content size changes.
 *
 * @param elementRef - Ref to the content element to observe
 * @param options - Auto-size options
 */
export function useAutoSize(
  elementRef: Ref<HTMLElement | null>,
  options?: UseAutoSizeOptions
): UseAutoSizeReturn {
  const width = ref(0)
  const height = ref(0)
  let observer: ResizeObserver | null = null

  const isEnabled = (): boolean => {
    if (options?.enabled === undefined) return true
    if (typeof options.enabled === 'boolean') return options.enabled
    return options.enabled.value
  }

  const unwrap = (val: MaybeRef<number> | undefined): number | undefined => {
    if (val === undefined) return undefined
    if (typeof val === 'number') return val
    return val.value
  }

  const clamp = (value: number, min?: MaybeRef<number>, max?: MaybeRef<number>): number => {
    let result = value
    const minVal = unwrap(min)
    const maxVal = unwrap(max)
    if (minVal !== undefined) result = Math.max(result, minVal)
    if (maxVal !== undefined) result = Math.min(result, maxVal)
    return result
  }

  const updateSize = async (): Promise<void> => {
    if (!elementRef.value) return
    if (!isEnabled()) return

    // Use scrollWidth/scrollHeight to get natural content size
    const rawWidth = elementRef.value.scrollWidth
    const rawHeight = elementRef.value.scrollHeight

    // Check for scrollable child elements to calculate total content height
    const scrollableChild = elementRef.value.querySelector(
      '.overflow-y-auto, .overflow-auto, [style*="overflow-y: auto"], [style*="overflow: auto"]'
    ) as HTMLElement | null

    // Calculate total content height including scrollable content
    let totalContentHeight = rawHeight
    if (scrollableChild) {
      const hiddenHeight = scrollableChild.scrollHeight - scrollableChild.clientHeight
      if (hiddenHeight > 0) {
        totalContentHeight += hiddenHeight
      }
    }

    // Determine final height:
    // - If total content exceeds maxHeight, use maxHeight (enables scrolling)
    // - Otherwise, use actual content height (no scrolling needed)
    const maxVal = unwrap(options?.maxHeight)
    const newWidth = clamp(rawWidth, options?.minWidth, options?.maxWidth)
    const newHeight = maxVal && totalContentHeight > maxVal
      ? maxVal // Content exceeds max, fix at maxHeight to enable scrolling
      : clamp(rawHeight, options?.minHeight, options?.maxHeight)

    // Skip if size hasn't changed
    if (newWidth === width.value && newHeight === height.value) return
    if (newWidth === 0 || newHeight === 0) return

    width.value = newWidth
    height.value = newHeight

    try {
      await invoke('set_window_size', {
        width: newWidth,
        height: newHeight,
      })
    } catch (e) {
      console.error('Failed to set window size:', e)
    }
  }

  onMounted(async () => {
    if (elementRef.value && isEnabled()) {
      observer = new ResizeObserver(() => {
        updateSize()
      })
      observer.observe(elementRef.value)
      // Wait for DOM to be fully rendered before initial size update
      await nextTick()
      updateSize()
    }
  })

  onUnmounted(() => {
    if (observer) {
      observer.disconnect()
      observer = null
    }
  })

  return { width, height, updateSize }
}
