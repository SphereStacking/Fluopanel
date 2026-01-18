<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import type { ActiveAppProvider, ActiveAppInfo } from '@arcana/providers'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  provider: ActiveAppProvider
}>()

const activeApp = ref<ActiveAppInfo | null>(null)
const appIcon = ref<string | null>(null)
let unsubscribe: (() => void) | null = null
let intervalId: ReturnType<typeof setInterval> | null = null

const fetchIcon = async (appName: string) => {
  try {
    const result = await invoke<{ app: string; icon: string | null }>('get_app_icon', { app: appName })
    appIcon.value = result.icon
  } catch (error) {
    console.error('Failed to get app icon:', error)
    appIcon.value = null
  }
}

const refresh = async () => {
  try {
    const app = await props.provider.getActiveApp()
    if (app.name !== activeApp.value?.name) {
      activeApp.value = app
      if (app.name) {
        await fetchIcon(app.name)
      }
    }
  } catch (error) {
    console.error('Failed to refresh active app:', error)
  }
}

onMounted(async () => {
  await refresh()

  unsubscribe = props.provider.onActiveAppChange(async (app) => {
    if (app.name !== activeApp.value?.name) {
      activeApp.value = app
      if (app.name) {
        await fetchIcon(app.name)
      }
    }
  })

  intervalId = setInterval(refresh, 500)
})

onUnmounted(() => {
  unsubscribe?.()
  if (intervalId) clearInterval(intervalId)
})
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
