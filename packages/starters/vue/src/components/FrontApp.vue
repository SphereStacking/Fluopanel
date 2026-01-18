<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useActiveAppProvider } from '@arcana/vue'

const { data: activeApp } = useActiveAppProvider()
const appIcon = ref<string | null>(null)

const fetchIcon = async (appName: string) => {
  try {
    const result = await invoke<{ app: string; icon: string | null }>('get_app_icon', { app: appName })
    appIcon.value = result.icon
  } catch (error) {
    console.error('Failed to get app icon:', error)
    appIcon.value = null
  }
}

// Watch for app changes and fetch icon
watch(activeApp, (newApp, oldApp) => {
  if (newApp && newApp.name !== oldApp?.name) {
    fetchIcon(newApp.name)
  }
}, { immediate: true })
</script>

<template>
  <div
    v-if="activeApp"
    class="
      flex items-center gap-2 pl-2 pr-3 py-1
      rounded-lg
      transition-all duration-200
      hover:bg-[var(--widget-glass-hover)]
      group
    "
  >
    <!-- App icon with glass container -->
    <div
      v-if="appIcon"
      class="
        relative w-5 h-5
        rounded-md overflow-hidden
        ring-1 ring-white/10
        shadow-[0_2px_8px_rgba(0,0,0,0.3)]
        transition-transform duration-200
        group-hover:scale-110
      "
    >
      <img
        :src="`data:image/png;base64,${appIcon}`"
        :alt="activeApp.name"
        class="w-full h-full object-cover"
      />
    </div>

    <!-- App name with subtle glow on hover -->
    <span
      class="
        text-[13px] font-semibold tracking-wide
        text-[var(--text-primary)]
        transition-all duration-200
        group-hover:text-[var(--holo-cyan)]
      "
    >{{ activeApp.name }}</span>
  </div>
</template>
