<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { Icon } from '@iconify/vue'
import { usePopup } from '@arcana/vue'
import { useGitHub } from '../composables/useGitHub'

// GitHub data management
const github = useGitHub()

// Trigger refs for positioning
const issuesTriggerRef = ref<HTMLElement | null>(null)
const prsTriggerRef = ref<HTMLElement | null>(null)
const notificationsTriggerRef = ref<HTMLElement | null>(null)

// Popup management - separate popup for each type
const issuesPopup = usePopup({
  width: 340,
  height: 420,
  align: 'center',
  offsetY: 8,
})
const prsPopup = usePopup({
  width: 340,
  height: 420,
  align: 'center',
  offsetY: 8,
})
// Notifications uses hover mode with Rust-side trigger monitoring
// triggerRef enables global mouse detection even when window is not focused
const notificationsPopup = usePopup({
  mode: 'hover',
  width: 340,
  height: 420,
  align: 'center',
  offsetY: 8,
  triggerRef: notificationsTriggerRef,
  triggerId: 'github-notifications',
})

// Click handlers
async function openIssues() {
  if (issuesTriggerRef.value) {
    await issuesPopup.toggle('github-issues', issuesTriggerRef.value)
  }
}

async function openPRs() {
  if (prsTriggerRef.value) {
    await prsPopup.toggle('github-prs', prsTriggerRef.value)
  }
}

// Start/stop polling based on component lifecycle
onMounted(() => {
  github.startPolling()
})

onUnmounted(() => {
  github.stopPolling()
})

// Color states
const issueColor = computed(() => {
  if (!github.isConfigured.value) return 'text-text-ghost'
  if (github.issueCount.value > 0) return 'text-holo-cyan'
  return 'text-text-secondary'
})

const prColor = computed(() => {
  if (!github.isConfigured.value) return 'text-text-ghost'
  if (github.prCount.value > 0) return 'text-holo-purple'
  return 'text-text-secondary'
})

const notificationColor = computed(() => {
  if (!github.isConfigured.value) return 'text-text-ghost'
  if (github.notificationCount.value > 0) return 'text-holo-yellow'
  return 'text-text-secondary'
})
</script>

<template>
  <div class="flex items-center gap-0.5">
    <!-- My Issues -->
    <div
      ref="issuesTriggerRef"
      @click="openIssues"
      class="flex items-center gap-1 py-1 px-2 rounded-lg text-[12px] tracking-wide transition-all duration-200 hover:bg-widget-glass-hover cursor-pointer group"
    >
      <Icon
        icon="octicon:issue-opened-16"
        class="w-[14px] h-[14px] transition-colors duration-200"
        :class="issueColor"
      />
      <span
        v-if="github.issueCount.value > 0"
        class="font-medium tabular-nums text-text-secondary group-hover:text-text-primary transition-colors duration-200"
        >{{ github.issueCount.value }}</span
      >
    </div>

    <!-- My Pull Requests -->
    <div
      ref="prsTriggerRef"
      @click="openPRs"
      class="flex items-center gap-1 py-1 px-2 rounded-lg text-[12px] tracking-wide transition-all duration-200 hover:bg-widget-glass-hover cursor-pointer group"
    >
      <Icon
        icon="octicon:git-pull-request-16"
        class="w-[14px] h-[14px] transition-colors duration-200"
        :class="prColor"
      />
      <span
        v-if="github.prCount.value > 0"
        class="font-medium tabular-nums text-text-secondary group-hover:text-text-primary transition-colors duration-200"
        >{{ github.prCount.value }}</span
      >
    </div>

    <!-- Unread Notifications (hover mode with Rust-side trigger monitoring) -->
    <div
      ref="notificationsTriggerRef"
      class="flex items-center gap-1 py-1 px-2 rounded-lg text-[12px] tracking-wide transition-all duration-200 cursor-pointer group"
      :class="notificationsPopup.isOpen.value ? 'bg-widget-glass-hover' : 'hover:bg-widget-glass-hover'"
    >
      <Icon
        icon="octicon:inbox-16"
        class="w-[14px] h-[14px] transition-colors duration-200"
        :class="notificationColor"
      />
      <span
        v-if="github.notificationCount.value > 0"
        class="font-medium tabular-nums text-text-secondary group-hover:text-text-primary transition-colors duration-200"
        >{{ github.notificationCount.value }}</span
      >
    </div>
  </div>
</template>
