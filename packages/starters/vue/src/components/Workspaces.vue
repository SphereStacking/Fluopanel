<script setup lang="ts">
import { computed } from 'vue'
import type { Workspace } from '@arcana/providers'
import { useAerospaceProvider } from '@arcana/vue'

const MAX_ICONS = 3

const { workspaces, focusWorkspace, getAppIcon } = useAerospaceProvider()

const displayWorkspaces = computed((): Workspace[] => {
  return workspaces.value
    .filter(ws => ws.windows.length > 0)
    .sort((a, b) => a.id.localeCompare(b.id, undefined, { numeric: true }))
})

const handleClick = async (id: string) => {
  try {
    await focusWorkspace(id)
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
