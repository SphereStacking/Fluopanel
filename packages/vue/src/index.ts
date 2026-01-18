// Components
export { default as Widget } from './Widget.vue'
export { default as Popup } from './Popup.vue'

// Composables
export { useWidgetMode, useCoordinator } from './composables/useWidgetMode'
export { usePopup, usePopupMode, type UsePopupOptions, type UsePopupReturn, type PopupAnchor, type PopupAlign } from './composables/usePopup'
export { useSharedStore, type UseSharedStoreReturn } from './composables/useSharedStore'

// Provider composables
export {
  useCpuProvider,
  useMemoryProvider,
  useNetworkProvider,
  useBatteryProvider,
  useVolumeProvider,
  useBrightnessProvider,
  useDiskProvider,
  useMediaProvider,
  useYouTubeMusicProvider,
  useActiveAppProvider,
  useAerospaceProvider,
  useDateProvider,
  useBluetoothProvider,
} from './composables/providers'
