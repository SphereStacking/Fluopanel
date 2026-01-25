<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { open } from '@tauri-apps/plugin-shell'
import { Icon } from '@iconify/vue'
import { useGitHub, type GitHubItem } from '../../composables/useGitHub'

const github = useGitHub()

// Section definitions with visibility computed
const sections = computed(() => [
  {
    key: 'opened',
    label: 'Opened',
    items: github.issues.value.opened,
    show: github.issues.value.opened.length > 0,
  },
  {
    key: 'assigned',
    label: 'Assigned',
    items: github.issues.value.assigned,
    show: github.issues.value.assigned.length > 0,
  },
  {
    key: 'closed',
    label: 'Recently Closed',
    items: github.issues.value.recentlyClosed,
    show: github.issues.value.recentlyClosed.length > 0,
  },
])

const visibleSections = computed(() => sections.value.filter((s) => s.show))
const totalCount = computed(() => github.issueCount.value)
const hasAnyItems = computed(() => visibleSections.value.length > 0)

async function openItem(item: GitHubItem) {
  await open(item.html_url)
}

async function openGitHubIssues() {
  await open('https://github.com/issues')
}

onMounted(() => {
  github.fetchIfStale()
})
</script>

<template>
  <div
    class="relative h-full font-[-apple-system,BlinkMacSystemFont,'SF_Pro_Text',sans-serif] text-[13px] antialiased"
  >
    <!-- Glass background -->
    <div
      class="absolute inset-0 rounded-xl border border-glass-border bg-glass-bg backdrop-blur-[40px] backdrop-saturate-[180%]"
    />

    <div class="relative flex flex-col h-full rounded-xl overflow-hidden">
      <!-- Header -->
      <header class="flex items-center justify-between px-3.5 py-3 border-b border-white/[0.04]">
        <div class="flex items-center gap-2">
          <Icon icon="octicon:issue-opened-16" class="w-4 h-4 text-holo-cyan" />
          <span class="text-text-primary font-semibold tracking-tight">Issues</span>
        </div>
        <span
          v-if="totalCount > 0"
          class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-holo-cyan/12 text-holo-cyan border border-holo-cyan/20"
        >
          {{ totalCount }}
        </span>
      </header>

      <!-- Content -->
      <div class="flex-1 min-h-0 overflow-y-auto popup-scrollbar">
        <!-- Loading -->
        <div
          v-if="github.loading.value"
          class="flex flex-col items-center justify-center h-32 gap-2"
        >
          <Icon icon="mdi:loading" class="w-5 h-5 animate-spin text-text-tertiary" />
          <span class="text-[11px] text-text-tertiary">Loading issues...</span>
        </div>

        <!-- Error -->
        <div
          v-else-if="github.error.value"
          class="flex flex-col items-center justify-center h-32 gap-2"
        >
          <Icon icon="mdi:alert-circle" class="w-5 h-5 text-danger" />
          <span class="text-[11px] text-text-tertiary">{{ github.error.value }}</span>
        </div>

        <!-- Empty -->
        <div
          v-else-if="!hasAnyItems"
          class="flex flex-col items-center justify-center h-32 gap-2"
        >
          <Icon icon="octicon:issue-opened-16" class="w-6 h-6 text-text-ghost" />
          <span class="text-[11px] text-text-tertiary">No issues</span>
        </div>

        <!-- Sections -->
        <div v-else class="py-1">
          <template v-for="(section, sectionIndex) in visibleSections" :key="section.key">
            <!-- Section Header -->
            <div
              class="flex items-center gap-2 px-3.5 py-1.5 animate-fade-in"
              :class="{ 'mt-1': sectionIndex > 0 }"
              :style="{ animationDelay: `${sectionIndex * 50}ms` }"
            >
              <span class="text-[10px] font-semibold uppercase tracking-wider text-text-ghost">
                {{ section.label }}
              </span>
              <span class="text-[10px] text-text-ghost/60">({{ section.items.length }})</span>
              <div class="flex-1 h-px bg-white/[0.04]" />
            </div>

            <!-- Items -->
            <div
              v-for="(item, itemIndex) in section.items"
              :key="item.id"
              @click="openItem(item)"
              class="group flex items-center gap-2.5 mx-1.5 px-2 py-1.5 rounded-lg cursor-pointer transition-all duration-100 hover:bg-widget-glass-hover active:bg-widget-glass active:scale-[0.98] animate-fade-slide-in"
              :style="{ animationDelay: `${sectionIndex * 50 + itemIndex * 25}ms` }"
            >
              <!-- Status dot -->
              <span
                class="w-1.5 h-1.5 rounded-full flex-shrink-0 transition-all duration-150"
                :class="
                  section.key === 'closed'
                    ? 'bg-text-ghost group-hover:bg-text-tertiary'
                    : 'bg-holo-cyan/70 group-hover:bg-holo-cyan group-hover:shadow-[0_0_6px_var(--holo-cyan)]'
                "
              />

              <!-- Content -->
              <div class="flex-1 min-w-0">
                <span
                  class="block truncate text-[12px] leading-tight transition-colors duration-100"
                  :class="
                    section.key === 'closed'
                      ? 'text-text-tertiary group-hover:text-text-secondary'
                      : 'text-text-primary group-hover:text-white'
                  "
                >
                  {{ item.title }}
                </span>
              </div>

              <!-- Arrow -->
              <Icon
                icon="mdi:chevron-right"
                class="w-3 h-3 text-text-ghost opacity-0 -translate-x-1 transition-all duration-100 group-hover:opacity-60 group-hover:translate-x-0"
              />
            </div>
          </template>
        </div>
      </div>

      <!-- Footer -->
      <footer class="px-2.5 py-2 border-t border-white/[0.04]">
        <button
          @click="openGitHubIssues"
          class="w-full flex items-center justify-center gap-1.5 py-2 rounded-lg bg-widget-glass border border-transparent text-text-secondary text-[11px] font-medium cursor-pointer transition-all duration-150 hover:bg-widget-glass-hover hover:text-text-primary hover:border-holo-cyan/20 group"
        >
          <span>View All Issues</span>
          <Icon
            icon="mdi:chevron-right"
            class="w-3.5 h-3.5 transition-transform duration-150 group-hover:translate-x-0.5"
          />
        </button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
@keyframes fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes fade-slide-in {
  from {
    opacity: 0;
    transform: translateY(3px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-fade-in {
  animation: fade-in 0.2s ease forwards;
  opacity: 0;
}

.animate-fade-slide-in {
  animation: fade-slide-in 0.2s ease forwards;
  opacity: 0;
}

/* Custom scrollbar */
.popup-scrollbar::-webkit-scrollbar {
  width: 5px;
}

.popup-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.popup-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.08);
  border-radius: 2.5px;
}

.popup-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.12);
}
</style>
