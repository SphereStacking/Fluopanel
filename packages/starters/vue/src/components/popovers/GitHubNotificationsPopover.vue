<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { open } from '@tauri-apps/plugin-shell'
import { Icon } from '@iconify/vue'
import { useGitHub, type GitHubNotification } from '../../composables/useGitHub'

const github = useGitHub()

const notifications = computed(() => github.notifications.value)
const unreadCount = computed(() => github.notificationCount.value)
const hasNotifications = computed(() => notifications.value.length > 0)

// Group by unread status
const unreadNotifications = computed(() => notifications.value.filter((n) => n.unread))
const readNotifications = computed(() => notifications.value.filter((n) => !n.unread))

async function openNotification(notification: GitHubNotification) {
  if (notification.unread) {
    await github.markNotificationRead(notification.id)
  }
  await open(github.getNotificationUrl(notification))
}

async function openGitHubNotifications() {
  await open('https://github.com/notifications')
}

// Map notification type to icon name
function getTypeIcon(type: string): string {
  switch (type) {
    case 'Issue':
      return 'octicon:issue-opened-16'
    case 'PullRequest':
      return 'octicon:git-pull-request-16'
    case 'Discussion':
      return 'octicon:comment-discussion-16'
    case 'Release':
      return 'octicon:tag-16'
    case 'Commit':
      return 'octicon:git-commit-16'
    default:
      return 'octicon:inbox-16'
  }
}

onMounted(() => {
  github.fetchIfStale()
})
</script>

<template>
  <div
    class="relative font-[-apple-system,BlinkMacSystemFont,'SF_Pro_Text',sans-serif] text-[13px] antialiased"
  >
    <!-- Glass background -->
    <div
      class="absolute inset-0 rounded-xl border border-glass-border bg-glass-bg  backdrop-saturate-[180%] "
    />

    <div class="relative flex flex-col rounded-xl overflow-hidden">
      <!-- Header -->
      <header class="flex items-center justify-between px-3.5 py-3 border-b border-white/[0.04]">
        <div class="flex items-center gap-2">
          <Icon icon="octicon:inbox-16" class="w-4 h-4 text-holo-yellow" />
          <span class="text-text-primary font-semibold tracking-tight">Inbox</span>
        </div>
        <span
          v-if="unreadCount > 0"
          class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-holo-yellow/12 text-holo-yellow border border-holo-yellow/20"
        >
          {{ unreadCount }} new
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
          <span class="text-[11px] text-text-tertiary">Loading notifications...</span>
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
          v-else-if="!hasNotifications"
          class="flex flex-col items-center justify-center h-32 gap-2"
        >
          <Icon icon="octicon:check-circle-16" class="w-6 h-6 text-text-ghost" />
          <span class="text-[11px] text-text-tertiary">All caught up!</span>
        </div>

        <!-- Notification List -->
        <div v-else class="py-1">
          <!-- Unread Section -->
          <template v-if="unreadNotifications.length > 0">
            <div class="flex items-center gap-2 px-3.5 py-1.5 animate-fade-in">
              <span class="text-[10px] font-semibold uppercase tracking-wider text-text-ghost">
                Unread
              </span>
              <span class="text-[10px] text-text-ghost/60">({{ unreadNotifications.length }})</span>
              <div class="flex-1 h-px bg-white/[0.04]" />
            </div>

            <div
              v-for="(notification, index) in unreadNotifications"
              :key="notification.id"
              @click="openNotification(notification)"
              class="group flex items-center gap-2.5 mx-1.5 px-2 py-1.5 rounded-lg cursor-pointer transition-all duration-100 hover:bg-widget-glass-hover active:bg-widget-glass active:scale-[0.98] animate-fade-slide-in"
              :style="{ animationDelay: `${index * 25}ms` }"
            >
              <!-- Type icon with unread indicator -->
              <div class="relative flex-shrink-0">
                <Icon :icon="getTypeIcon(notification.subject.type)" class="w-3.5 h-3.5 text-holo-yellow" />
                <span
                  class="absolute -top-0.5 -right-0.5 w-1.5 h-1.5 rounded-full bg-holo-yellow shadow-[0_0_6px_var(--holo-yellow)]"
                />
              </div>

              <!-- Content -->
              <div class="flex-1 min-w-0">
                <span class="block truncate text-[12px] leading-tight text-text-primary group-hover:text-white transition-colors duration-100">
                  {{ notification.subject.title }}
                </span>
              </div>

              <!-- Arrow -->
              <Icon
                icon="mdi:chevron-right"
                class="w-3 h-3 text-text-ghost opacity-0 -translate-x-1 transition-all duration-100 group-hover:opacity-60 group-hover:translate-x-0"
              />
            </div>
          </template>

          <!-- Read Section -->
          <template v-if="readNotifications.length > 0">
            <div
              class="flex items-center gap-2 px-3.5 py-1.5 animate-fade-in"
              :class="{ 'mt-1': unreadNotifications.length > 0 }"
              :style="{ animationDelay: `${unreadNotifications.length * 25 + 50}ms` }"
            >
              <span class="text-[10px] font-semibold uppercase tracking-wider text-text-ghost">
                Earlier
              </span>
              <span class="text-[10px] text-text-ghost/60">({{ readNotifications.length }})</span>
              <div class="flex-1 h-px bg-white/[0.04]" />
            </div>

            <div
              v-for="(notification, index) in readNotifications"
              :key="notification.id"
              @click="openNotification(notification)"
              class="group flex items-center gap-2.5 mx-1.5 px-2 py-1.5 rounded-lg cursor-pointer transition-all duration-100 hover:bg-widget-glass-hover active:bg-widget-glass active:scale-[0.98] animate-fade-slide-in"
              :style="{ animationDelay: `${unreadNotifications.length * 25 + 50 + index * 25}ms` }"
            >
              <!-- Type icon -->
              <Icon
                :icon="getTypeIcon(notification.subject.type)"
                class="w-3.5 h-3.5 text-text-ghost flex-shrink-0 group-hover:text-text-tertiary transition-colors duration-100"
              />

              <!-- Content -->
              <div class="flex-1 min-w-0">
                <span class="block truncate text-[12px] leading-tight text-text-tertiary group-hover:text-text-secondary transition-colors duration-100">
                  {{ notification.subject.title }}
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
          @click="openGitHubNotifications"
          class="w-full flex items-center justify-center gap-1.5 py-2 rounded-lg bg-widget-glass border border-transparent text-text-secondary text-[11px] font-medium cursor-pointer transition-all duration-150 hover:bg-widget-glass-hover hover:text-text-primary hover:border-holo-yellow/20 group"
        >
          <span>View All Notifications</span>
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
