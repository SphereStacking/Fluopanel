export * from './types'
export * from './aerospace'
export * from './battery'
export * from './cpu'
export * from './memory'
export * from './network'
export * from './date'
export * from './media'
export * from './volume'
export * from './activeApp'
export * from './disk'
export * from './brightness'
export * from './bluetooth'

import { createAerospaceProvider } from './aerospace'
import { createBatteryProvider } from './battery'
import { createCpuProvider } from './cpu'
import { createMemoryProvider } from './memory'
import { createNetworkProvider } from './network'
import { createDateProvider } from './date'
import { createMediaProvider } from './media'
import { createVolumeProvider } from './volume'
import { createActiveAppProvider } from './activeApp'
import { createDiskProvider } from './disk'
import { createBrightnessProvider } from './brightness'
import { createBluetoothProvider } from './bluetooth'

export function createProviders() {
  return {
    aerospace: createAerospaceProvider(),
    battery: createBatteryProvider(),
    cpu: createCpuProvider(),
    memory: createMemoryProvider(),
    network: createNetworkProvider(),
    date: createDateProvider(),
    media: createMediaProvider(),
    volume: createVolumeProvider(),
    activeApp: createActiveAppProvider(),
    disk: createDiskProvider(),
    brightness: createBrightnessProvider(),
    bluetooth: createBluetoothProvider(),
  }
}
