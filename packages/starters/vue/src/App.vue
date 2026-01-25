<script setup lang="ts">
import { ref } from 'vue'
import { Window, useCoordinator, Popover, usePopoverMode, usePopover } from '@arcana/vue'

// Settings menu popover
const settingsMenuRef = ref<HTMLElement | null>(null)
const settingsPopover = usePopover({
  width: 200,
  height: 190,
  align: 'start',
  offsetY: 8,
})

const toggleSettingsMenu = () => {
  if (settingsMenuRef.value) {
    settingsPopover.toggle('settings-menu', settingsMenuRef.value)
  }
}

// Bar components
import Workspaces from './components/Workspaces.vue'
import FrontApp from './components/FrontApp.vue'
import Clock from './components/Clock.vue'
import Battery from './components/Battery.vue'
import Cpu from './components/Cpu.vue'
import Memory from './components/Memory.vue'
import Media from './components/Media.vue'
import Disk from './components/Disk.vue'
import Bluetooth from './components/Bluetooth.vue'
import TestPopover from './components/TestPopover.vue'
import YouTubeMusic from './components/YouTubeMusic.vue'

// Popover components
import GitHubPRsPopover from './components/popovers/GitHubPRsPopover.vue'
import GitHubNotificationsPopover from './components/popovers/GitHubNotificationsPopover.vue'
import TestPopoverContent from './components/popovers/TestPopoverContent.vue'
import SettingsMenuPopover from './components/popovers/SettingsMenuPopover.vue'

const { isPopover } = usePopoverMode()

// Coordinator mode: auto-hide the main window after windows are created
// Don't auto-hide if this is a popover window (popover needs to stay visible)
useCoordinator({ autoHide: !isPopover.value })
</script>

<template>
  <!-- Main bar window -->
  <Window
    id="bar"
    :position="{ top: 9, left: 20, right: 20, height: 60 }"
  >
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
          <!-- Apple Logo - Settings Menu -->
          <div
            ref="settingsMenuRef"
            class="
              flex items-center justify-center w-7 h-7
              rounded-lg cursor-pointer
              transition-all duration-300
              hover:bg-[var(--widget-glass-hover)]
              group
            "
            @click="toggleSettingsMenu"
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
          <TestPopover popoverId="test-popover1" />
          <TestPopover popoverId="test-popover2" />
          <div class="w-px h-4 bg-[var(--glass-border)] mx-1" />
          <Disk />
          <Cpu />
          <Bluetooth />
          <Memory />
          <Battery />
        </div>
      </div>
    </div>
  </Window>

  <!-- GitHub Popovers -->
  <Popover id="github-prs">
    <GitHubPRsPopover />
  </Popover>
  <Popover id="github-notifications">
    <GitHubNotificationsPopover />
  </Popover>

  <!-- Test Popovers -->
  <Popover id="test-popover1">
    <TestPopoverContent />
  </Popover>
  <Popover id="test-popover2">
    <TestPopoverContent />
  </Popover>

  <!-- Settings Menu Popover -->
  <Popover id="settings-menu">
    <SettingsMenuPopover />
  </Popover>
</template>

<style scoped>
@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}
</style>
