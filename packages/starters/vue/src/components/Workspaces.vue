<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import type { AerospaceProvider, Workspace, FocusChangeEvent } from '@arcana/providers'

const props = defineProps<{
  provider: AerospaceProvider
}>()

const MAX_ICONS = 3

const workspaces = ref<Workspace[]>([])
const appIcons = ref<Map<string, string | null>>(new Map())
let unsubscribeFocus: (() => void) | null = null
let unsubscribeWorkspace: (() => void) | null = null

// Show workspaces 1-9, merge data with empty slots
const displayWorkspaces = computed((): Workspace[] => {
  const slots: Workspace[] = Array.from({ length: 9 }, (_, i) => ({
    id: String(i + 1),
    displayName: String(i + 1),
    focused: false,
    visible: false,
    windows: [],
    monitor: 0,
  }))

  for (const ws of workspaces.value) {
    const index = parseInt(ws.id, 10) - 1
    if (index >= 0 && index < 9) {
      slots[index] = ws
    }
  }

  return slots
})

const refresh = async () => {
  try {
    const result = await props.provider.getWorkspaces()
    workspaces.value = result

    const appNames = new Set<string>()
    result.forEach(ws => ws.windows.forEach(w => appNames.add(w.app)))

    if (appNames.size > 0) {
      const icons = await props.provider.getAppIcons([...appNames])
      icons.forEach(icon => {
        appIcons.value.set(icon.app, icon.icon)
      })
    }
  } catch (error) {
    console.error('Failed to refresh workspaces:', error)
  }
}

const getAppIcon = (appName: string): string | null => {
  return appIcons.value.get(appName) ?? null
}

onMounted(async () => {
  // Initial load
  await refresh()

  // Subscribe to focus change events (optimized: only 2 workspaces updated)
  unsubscribeFocus = props.provider.onFocusChange(async ({ focused, prev }: FocusChangeEvent) => {
    // Update only the changed workspaces in cache
    workspaces.value = workspaces.value.map((ws) => {
      if (focused && ws.id === focused.id) return focused
      if (prev && ws.id === prev.id) return { ...prev, focused: false }
      return { ...ws, focused: false }
    })

    // Fetch icons for any new apps
    const newApps = new Set<string>()
    if (focused) focused.windows.forEach((w) => newApps.add(w.app))
    if (prev) prev.windows.forEach((w) => newApps.add(w.app))

    const uncached = [...newApps].filter((app) => !appIcons.value.has(app))
    if (uncached.length > 0) {
      const icons = await props.provider.getAppIcons(uncached)
      icons.forEach((icon) => {
        appIcons.value.set(icon.app, icon.icon)
      })
    }
  })

  // Subscribe to full workspace change events (for window moves)
  unsubscribeWorkspace = props.provider.onWorkspaceChange(async (ws) => {
    workspaces.value = ws

    // Update icons for any new apps
    const appNames = new Set<string>()
    ws.forEach((w) => w.windows.forEach((win) => appNames.add(win.app)))

    const uncached = [...appNames].filter((app) => !appIcons.value.has(app))
    if (uncached.length > 0) {
      const icons = await props.provider.getAppIcons(uncached)
      icons.forEach((icon) => {
        appIcons.value.set(icon.app, icon.icon)
      })
    }
  })
})

onUnmounted(() => {
  unsubscribeFocus?.()
  unsubscribeWorkspace?.()
})

const handleClick = async (id: string) => {
  try {
    await props.provider.focusWorkspace(id)
    await refresh()
  } catch (error) {
    console.error('Failed to focus workspace:', error)
  }
}

const getUniqueApps = (ws: Workspace): string[] => {
  const seen = new Set<string>()
  return ws.windows
    .filter(w => {
      if (seen.has(w.app)) return false
      seen.add(w.app)
      return true
    })
    .map(w => w.app)
}
</script>

<template>
  <!-- Workspace container -->
  <div class="flex items-center gap-0.5 px-1 py-0.5 rounded-lg">
    <div
      v-for="ws in displayWorkspaces"
      :key="ws.id"
      class="
        relative flex items-center gap-1.5 px-2 py-0.5 rounded-md cursor-pointer
        text-[13px] tracking-wide
        transition-all duration-200 ease-out
        group
      "
      :class="[
        ws.focused
          ? 'bg-[var(--accent)]/15 text-[var(--accent)]'
          : ws.windows.length > 0
            ? 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--widget-glass-hover)]'
            : 'text-[var(--text-ghost)] hover:text-[var(--text-tertiary)] hover:bg-[var(--widget-glass-hover)]'
      ]"
      @click="handleClick(ws.id)"
    >
      <!-- Workspace number -->
      <span
        class="font-semibold min-w-[0.75rem] text-center"
        :class="{ 'holo-text': ws.focused }"
      >{{ ws.id }}</span>

      <!-- App icons with glassmorphism container -->
      <template v-if="ws.windows.length > 0">
        <div class="flex items-center gap-0.5">
          <template v-for="app in getUniqueApps(ws).slice(0, MAX_ICONS)" :key="app">
            <div
              v-if="getAppIcon(app)"
              class="
                relative w-4 h-4
                rounded-[4px] overflow-hidden
                ring-1 ring-white/10
                transition-transform duration-200
                group-hover:scale-110
              "
            >
              <img
                :src="`data:image/png;base64,${getAppIcon(app)}`"
                :alt="app"
                :title="app"
                class="w-full h-full object-cover"
              />
            </div>
          </template>
          <span
            v-if="getUniqueApps(ws).length > MAX_ICONS"
            class="text-[10px] text-[var(--text-ghost)] ml-0.5 font-medium"
          >
            +{{ getUniqueApps(ws).length - MAX_ICONS }}
          </span>
        </div>
      </template>
    </div>
  </div>
</template>
