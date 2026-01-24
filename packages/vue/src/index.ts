// Components
export { default as Window } from './Window.vue'
export { default as Popover } from './Popover.vue'

// Composables
export { useWindowMode, useCoordinator } from './composables/useWindowMode'
export { usePopover, usePopoverMode, type UsePopoverOptions, type UsePopoverReturn, type PopoverAnchor, type PopoverAlign } from './composables/usePopover'
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
