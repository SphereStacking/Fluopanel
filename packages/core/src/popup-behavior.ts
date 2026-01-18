import type { PopupMode, PopupBehavior } from './types'

/**
 * Popup behavior configurations by mode
 *
 * - toggle: Click to open/close, blur to close (default)
 * - hover: Mouseenter to open, mouseleave to close (tooltip-style)
 * - hover-sticky: Mouseenter to open, blur to close (menu-style)
 */
export const POPUP_BEHAVIORS: Record<PopupMode, PopupBehavior> = {
  toggle: {
    openOn: 'click',
    closeOn: 'blur',
    leaveDelay: 0,
  },
  hover: {
    openOn: 'mouseenter',
    closeOn: 'mouseleave',
    leaveDelay: 150,
  },
  'hover-sticky': {
    openOn: 'mouseenter',
    closeOn: 'blur',
    leaveDelay: 150,
  },
}

/**
 * Get popup behavior configuration for a given mode
 */
export function getPopupBehavior(mode: PopupMode): PopupBehavior {
  return POPUP_BEHAVIORS[mode]
}
