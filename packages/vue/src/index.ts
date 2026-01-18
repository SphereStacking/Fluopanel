// Components
export { default as Widget } from './Widget.vue'
export { default as Popup } from './Popup.vue'

// Composables
export { useWidgetMode, useCoordinator } from './composables/useWidgetMode'
export { usePopup, usePopupMode, type UsePopupOptions, type UsePopupReturn, type PopupAnchor, type PopupAlign, type PopupMode } from './composables/usePopup'
export { useSharedStore, type UseSharedStoreReturn } from './composables/useSharedStore'
export { useTrigger, type UseTriggerOptions, type UseTriggerReturn, type TriggerBounds } from './composables/useTrigger'
