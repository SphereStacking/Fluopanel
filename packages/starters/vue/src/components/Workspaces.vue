<script setup lang="ts">
import { computed } from 'vue'
import type { Workspace } from 'fluopanel-providers'
import { useAerospaceProvider } from 'fluopanel-vue'

interface Props {
  direction?: 'horizontal' | 'vertical'
}
const props = withDefaults(defineProps<Props>(), {
  direction: 'horizontal'
})

const isVertical = computed(() => props.direction === 'vertical')
const maxIcons = computed(() => isVertical.value ? 2 : 3)

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
  <!-- Workspace navigation -->
  <nav
    class="flex rounded-lg"
    :class="isVertical
      ? 'flex-col items-center gap-1'
      : 'items-center gap-0.5 px-1 py-0.5'"
  >
    <button
      v-for="ws in displayWorkspaces"
      :key="ws.id"
      type="button"
      class="relative flex cursor-pointer rounded-md transition-all duration-200 ease-out group"
      :class="isVertical
        ? [
            'flex-col items-center gap-0.5 p-1',
            ws.focused
              ? 'bg-[var(--accent)]/15'
              : 'hover:bg-[var(--widget-glass-hover)]'
          ]
        : [
            'items-center gap-1.5 px-2 py-0.5 text-[13px] tracking-wide',
            ws.focused
              ? 'bg-[var(--accent)]/15 text-[var(--accent)]'
              : ws.windows.length > 0
                ? 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--widget-glass-hover)]'
                : 'text-[var(--text-ghost)] hover:text-[var(--text-tertiary)] hover:bg-[var(--widget-glass-hover)]'
          ]"
      :title="isVertical ? `Workspace ${ws.id} (${ws.windows.length} windows)` : undefined"
      @click="handleClick(ws.id)"
    >
      <!-- Workspace number -->
      <span
        :class="isVertical
          ? [
              'text-[11px] font-semibold leading-none',
              ws.focused
                ? 'text-[var(--accent)] holo-text'
                : 'text-[var(--text-secondary)] group-hover:text-[var(--text-primary)]'
            ]
          : [
              'font-semibold min-w-[0.75rem] text-center',
              { 'holo-text': ws.focused }
            ]"
      >{{ ws.id }}</span>

      <!-- App icons -->
      <div
        v-if="ws.windows.length > 0"
        class="flex"
        :class="isVertical ? 'flex-col items-center gap-0.5' : 'items-center gap-0.5'"
      >
        <template v-for="app in getUniqueApps(ws).slice(0, maxIcons)" :key="app">
          <div
            v-if="getAppIcon(app)"
            class="
              rounded-[4px] overflow-hidden
              ring-1 ring-white/10
              transition-transform duration-200
              group-hover:scale-110
            "
            :class="isVertical ? 'w-5 h-5' : 'w-4 h-4'"
          >
            <img
              :src="`data:image/png;base64,${getAppIcon(app)}`"
              :alt="app"
              :title="isVertical ? undefined : app"
              class="w-full h-full object-cover"
            />
          </div>
        </template>
        <span
          v-if="getUniqueApps(ws).length > maxIcons"
          class="text-[var(--text-ghost)] font-medium"
          :class="isVertical ? 'text-[9px]' : 'text-[10px] ml-0.5'"
        >
          +{{ getUniqueApps(ws).length - maxIcons }}
        </span>
      </div>
    </button>
  </nav>
</template>
