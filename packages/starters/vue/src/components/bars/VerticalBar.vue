<script setup lang="ts">
import { useNativeMenu } from 'fluopanel-vue'
import { executeShell } from 'fluopanel-core'
import FluopanelIcon from '../icons/FluopanelIcon.vue'
import Workspaces from '../Workspaces.vue'
import ActiveApp from '../ActiveApp.vue'
import Clock from '../Clock.vue'
import GitHub from '../GitHub.vue'
import Media from '../Media.vue'
import YouTubeMusic from '../YouTubeMusic.vue'
import Battery from '../Battery.vue'
import Cpu from '../Cpu.vue'
import Memory from '../Memory.vue'
import Disk from '../Disk.vue'

// Native settings menu
const settingsMenu = useNativeMenu({
  items: [
    { id: 'fluopanel', label: '􀍟 Fluopanel 設定', accelerator: 'CmdOrCtrl+,' },
    { id: 'aerospace', label: '􀏜 Aerospace 設定' },
    { type: 'separator' },
    { id: 'zshrc', label: '􀪏 .zshrc' },
    { id: 'claude', label: '􀈕 .claude/' },
  ],
  onSelect: async (id) => {
    const commands: Record<string, string> = {
      fluopanel: 'code ~/.config/fluopanel/config.json',
      aerospace: 'code ~/.config/aerospace/aerospace.toml',
      zshrc: 'code ~/.zshrc',
      claude: 'code ~/.claude',
    }
    if (commands[id]) {
      await executeShell(commands[id])
    }
  },
})
</script>

<template>
  <!-- Vertical bar container with holographic glass effect -->
  <div
    class="
      relative flex flex-col items-center justify-between h-full w-10 py-3
      glass rounded-2xl
      overflow-hidden
    "
  >
    <!-- Holographic shimmer overlay (vertical) -->
    <div
      class="
        absolute inset-0 pointer-events-none
        bg-[linear-gradient(195deg,transparent_40%,rgba(255,255,255,0.03)_45%,rgba(255,255,255,0.05)_50%,rgba(255,255,255,0.03)_55%,transparent_60%)]
        bg-[length:100%_200%]
        animate-[shimmer-vertical_8s_ease-in-out_infinite]
      "
    />

    <!-- Subtle holographic edge glow -->
    <div
      class="
        absolute inset-0 pointer-events-none rounded-2xl
        bg-[radial-gradient(ellipse_50%_80%_at_-20%_50%,rgba(110,180,255,0.08),transparent)]
      "
    />

    <!-- Header: Settings + Workspaces + ActiveApp -->
    <header class="flex flex-col items-center z-10">
      <!-- Fluopanel Logo - Settings Menu -->
      <button
        type="button"
        class="
          flex items-center justify-center w-7 h-7 mb-2
          rounded-lg cursor-pointer
          transition-all duration-300
          hover:bg-[var(--widget-glass-hover)]
          group
        "
        @click="settingsMenu.popup()"
      >
        <FluopanelIcon
          class="
            w-4 h-4
            text-[var(--text-secondary)]
            group-hover:text-[var(--holo-cyan)]
            transition-colors duration-300
          "
        />
      </button>

      <!-- Separator -->
      <div class="w-5 h-px bg-white/10 mb-2" />

      <!-- Workspaces -->
      <Workspaces direction="vertical" />

      <!-- Active App -->
      <ActiveApp direction="vertical" class="mt-1" />
    </header>

    <!-- Main: Clock, Media -->
    <main class="flex flex-col items-center z-10">
      <Clock direction="vertical" />
      <Media direction="vertical" class="mt-1" />
      <YouTubeMusic direction="vertical" class="mt-1" />
    </main>

    <!-- Footer: GitHub + System Indicators -->
    <footer class="flex flex-col items-center z-10">
      <GitHub direction="vertical" />

      <!-- Separator -->
      <div class="w-5 h-px bg-white/10 my-2" />

      <!-- System indicators -->
      <aside class="flex flex-col items-center gap-1">
        <Disk direction="vertical" />
        <Cpu direction="vertical" />
        <Memory direction="vertical" />
        <Battery direction="vertical" />
      </aside>
    </footer>
  </div>
</template>

<style scoped>
@keyframes shimmer-vertical {
  0% { background-position: 0 200%; }
  100% { background-position: 0 -200%; }
}
</style>
