<script setup lang="ts">
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
import TestPopover from './TestPopover.vue'
import YouTubeMusic from './YouTubeMusic.vue'
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

        <Workspaces />
        <FrontApp />
      </div>

      <!-- Center section: Media & Clock -->
      <div class="absolute left-1/2 -translate-x-1/2 flex items-center gap-3 z-10">
        <Media />
        <YouTubeMusic />
        <Clock />
      </div>

      <!-- Right section: System indicators -->
      <div class="flex items-center gap-1 z-10">
        <!-- <GitHub /> -->
        <TestPopover />
        <TestPopover />
        <div class="w-px h-4 bg-[var(--glass-border)] mx-1" />
        <!-- <Volume /> -->
        <!-- <Brightness /> -->
        <!-- <Network /> -->
        <div class="w-px h-4 bg-[var(--glass-border)] mx-1" />
        <Disk />
        <Cpu />
        <Bluetooth />
        <Memory />
        <Battery />
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
