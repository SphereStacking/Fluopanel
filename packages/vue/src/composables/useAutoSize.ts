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
  let resizeObserver: ResizeObserver | null = null
  let mutationObserver: MutationObserver | null = null

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

    const el = elementRef.value

    // Temporarily remove height constraints to measure true content size
    const originalHeight = el.style.height
    const originalMaxHeight = el.style.maxHeight
    const originalOverflow = el.style.overflow
    el.style.height = 'auto'
    el.style.maxHeight = 'none'
    el.style.overflow = 'hidden' // Prevent scrollbar from affecting measurement

    // Measure true content size
    const rawWidth = el.scrollWidth
    const rawHeight = el.scrollHeight

    // Restore original styles
    el.style.height = originalHeight
    el.style.maxHeight = originalMaxHeight
    el.style.overflow = originalOverflow

    // Determine final height:
    // - If content exceeds maxHeight, use maxHeight (enables scrolling)
    // - Otherwise, use actual content height (no scrolling needed)
    const maxVal = unwrap(options?.maxHeight)
    const needsVerticalScroll = maxVal !== undefined && rawHeight > maxVal

    // When vertical scrolling is needed, add scrollbar width to accommodate it
    const scrollbarWidth = needsVerticalScroll ? 8 : 0 // Match CSS scrollbar width

    const newWidth = clamp(rawWidth + scrollbarWidth, options?.minWidth, options?.maxWidth)
    const newHeight = needsVerticalScroll
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
      // ResizeObserver for size changes
      resizeObserver = new ResizeObserver(() => {
        updateSize()
      })
      resizeObserver.observe(elementRef.value)

      // Also observe all children for size changes (handles content inside overflow:auto)
      const children = elementRef.value.querySelectorAll('*')
      children.forEach((child) => {
        resizeObserver?.observe(child)
      })

      // MutationObserver for DOM changes (items added/removed)
      mutationObserver = new MutationObserver(() => {
        // Re-observe new children
        if (elementRef.value && resizeObserver) {
          const newChildren = elementRef.value.querySelectorAll('*')
          newChildren.forEach((child) => {
            resizeObserver?.observe(child)
          })
        }
        updateSize()
      })
      mutationObserver.observe(elementRef.value, {
        childList: true,
        subtree: true,
      })

      // Wait for DOM to be fully rendered before initial size update
      await nextTick()
      updateSize()
    }
  })

  onUnmounted(() => {
    if (resizeObserver) {
      resizeObserver.disconnect()
      resizeObserver = null
    }
    if (mutationObserver) {
      mutationObserver.disconnect()
      mutationObserver = null
    }
  })

  return { width, height, updateSize }
}
