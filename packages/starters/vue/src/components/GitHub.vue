<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { Icon } from '@iconify/vue'
import { usePopover } from '@arcana/vue'
import { useGitHub } from '../composables/useGitHub'

// GitHub data management
const github = useGitHub()

// Trigger refs for positioning
const issuesTriggerRef = ref<HTMLElement | null>(null)
const prsTriggerRef = ref<HTMLElement | null>(null)
const notificationsTriggerRef = ref<HTMLElement | null>(null)

// Popover management - separate popover for each type
// exclusive: true ensures only one popover is open at a time
const issuesPopover = usePopover({
  width: 340,
  height: 420,
  align: 'center',
  offsetY: 8,
  exclusive: true,
})
const prsPopover = usePopover({
  width: 340,
  height: 420,
  align: 'center',
  offsetY: 8,
  exclusive: true,
})
const notificationsPopover = usePopover({
  width: 340,
  height: 420,
  align: 'center',
  offsetY: 8,
  exclusive: true,
})

// Click handlers
async function openIssues() {
  if (issuesTriggerRef.value) {
    await issuesPopover.toggle('github-issues', issuesTriggerRef.value)
  }
}

async function openPRs() {
  if (prsTriggerRef.value) {
    await prsPopover.toggle('github-prs', prsTriggerRef.value)
  }
}

async function openNotifications() {
  if (notificationsTriggerRef.value) {
    await notificationsPopover.toggle('github-notifications', notificationsTriggerRef.value)
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

    <!-- Unread Notifications -->
    <div
      ref="notificationsTriggerRef"
      @click="openNotifications"
      class="flex items-center gap-1 py-1 px-2 rounded-lg text-[12px] tracking-wide transition-all duration-200 cursor-pointer group hover:bg-widget-glass-hover"
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
