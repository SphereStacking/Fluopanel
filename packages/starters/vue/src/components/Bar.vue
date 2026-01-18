<script setup lang="ts">
import type {
  AerospaceProvider,
  BatteryProvider,
  CpuProvider,
  MemoryProvider,
  NetworkProvider,
  DateProvider,
  MediaProvider,
  VolumeProvider,
  ActiveAppProvider,
  DiskProvider,
  BrightnessProvider,
  BluetoothProvider,
} from '@arcana/providers'
import Workspaces from './Workspaces.vue'
import FrontApp from './FrontApp.vue'
import Clock from './Clock.vue'
import Battery from './Battery.vue'
import Cpu from './Cpu.vue'
import Memory from './Memory.vue'
import Network from './Network.vue'
import Media from './Media.vue'
import Volume from './Volume.vue'
import Disk from './Disk.vue'
import Brightness from './Brightness.vue'
import Bluetooth from './Bluetooth.vue'
import GitHub from './GitHub.vue'

interface Providers {
  aerospace: AerospaceProvider
  battery: BatteryProvider
  cpu: CpuProvider
  memory: MemoryProvider
  network: NetworkProvider
  date: DateProvider
  media: MediaProvider
  volume: VolumeProvider
  activeApp: ActiveAppProvider
  disk: DiskProvider
  brightness: BrightnessProvider
  bluetooth: BluetoothProvider
}

const props = defineProps<{
  providers: Providers
}>()
</script>

<template>
  <div class="">
    <!-- Main bar container with holographic glass effect -->
    <div
      class="
        relative flex justify-between items-center h-10 px-3
        glass rounded-2xl
        overflow-hidden
      "
    >
      <!-- Holographic shimmer overlay -->
      <div
        class="
          absolute inset-0 pointer-events-none py-2
          bg-[linear-gradient(105deg,transparent_40%,rgba(255,255,255,0.03)_45%,rgba(255,255,255,0.05)_50%,rgba(255,255,255,0.03)_55%,transparent_60%)]
          bg-[length:200%_100%]
          animate-[shimmer_8s_ease-in-out_infinite]
        "
      />

      <!-- Subtle holographic edge glow -->
      <div
        class="
          absolute inset-0 pointer-events-none rounded-2xl
          bg-[radial-gradient(ellipse_80%_50%_at_50%_-20%,rgba(110,180,255,0.08),transparent)]
        "
      />

      <!-- Left section: Logo, Workspaces, Active App -->
      <div class="flex items-center gap-2 z-10">
        <!-- Apple Logo with holographic effect -->
        <div
          class="
            flex items-center justify-center w-7 h-7
            rounded-lg cursor-pointer
            transition-all duration-300
            hover:bg-[var(--widget-glass-hover)]
            group
          "
        >
          <span
            class="
              icon-sf text-base
              text-[var(--text-secondary)]
              group-hover:text-[var(--holo-cyan)]
              transition-colors duration-300
            "
          >􀣺</span>
        </div>

        <div class="w-px h-4 bg-[var(--glass-border)]" />

        <Workspaces :provider="props.providers.aerospace" />
        <FrontApp :provider="props.providers.activeApp" />
      </div>

      <!-- Center section: Media & Clock -->
      <div class="absolute left-1/2 -translate-x-1/2 flex items-center gap-3 z-10">
        <Media :provider="props.providers.media" />
        <Clock :provider="props.providers.date" />
      </div>

      <!-- Right section: System indicators -->
      <div class="flex items-center gap-1 z-10">
        <GitHub />
        <Bluetooth :provider="props.providers.bluetooth" />
        <Volume :provider="props.providers.volume" />
        <Brightness :provider="props.providers.brightness" />
        <Network :provider="props.providers.network" />
        <div class="w-px h-4 bg-[var(--glass-border)] mx-1" />
        <Disk :provider="props.providers.disk" />
        <Cpu :provider="props.providers.cpu" />
        <Memory :provider="props.providers.memory" />
        <div class="w-px h-4 bg-[var(--glass-border)] mx-1" />
        <Battery :provider="props.providers.battery" />
      </div>
    </div>
  </div>
</template>

<style scoped>
@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}
</style>
