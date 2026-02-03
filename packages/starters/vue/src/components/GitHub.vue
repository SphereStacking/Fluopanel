<script setup lang="ts">
import { ref, computed } from 'vue'
import { Icon } from '@iconify/vue'
import { usePopover } from 'fluopanel-vue'
import { useGitHub } from '../composables/useGitHub'

interface Props {
  direction?: 'horizontal' | 'vertical'
}
const props = withDefaults(defineProps<Props>(), {
  direction: 'horizontal'
})

const isVertical = computed(() => props.direction === 'vertical')

// GitHub data management
const github = useGitHub()

// Trigger refs for positioning
const issuesTriggerRef = ref<HTMLElement | null>(null)
const prsTriggerRef = ref<HTMLElement | null>(null)
const notificationsTriggerRef = ref<HTMLElement | null>(null)

// Popover management - config changes based on direction
// Note: Currently popover always appears below anchor. offsetX not yet supported.
const popoverConfig = computed(() => ({
  width: 340,
  height: 420,
  align: isVertical.value ? 'start' as const : 'center' as const,
  offsetY: 8,
  exclusive: true,
}))

const issuesPopover = usePopover(popoverConfig.value)
const prsPopover = usePopover(popoverConfig.value)
const notificationsPopover = usePopover(popoverConfig.value)

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

// Color states
const issueColor = computed(() => {
  if (!github.isConfigured.value) return 'text-[var(--text-ghost)]'
  if (github.issueCount.value > 0) return 'text-[var(--holo-cyan)]'
  return 'text-[var(--text-secondary)]'
})

const prColor = computed(() => {
  if (!github.isConfigured.value) return 'text-[var(--text-ghost)]'
  if (github.prCount.value > 0) return 'text-[var(--holo-purple)]'
  return 'text-[var(--text-secondary)]'
})

const notificationColor = computed(() => {
  if (!github.isConfigured.value) return 'text-[var(--text-ghost)]'
  if (github.notificationCount.value > 0) return 'text-[var(--holo-yellow)]'
  return 'text-[var(--text-secondary)]'
})
</script>

<template>
  <nav
    class="flex items-center gap-1"
    :class="isVertical ? 'flex-col' : ''"
  >
    <!-- My Issues -->
    <button
      ref="issuesTriggerRef"
      type="button"
      @click="openIssues"
      class="relative p-1.5 rounded-lg transition-all duration-200 hover:bg-[var(--widget-glass-hover)] cursor-pointer group"
      title="Issues"
    >
      <Icon
        icon="octicon:issue-opened-16"
        class="w-4 h-4 transition-colors duration-200"
        :class="issueColor"
      />
      <!-- Badge -->
      <span
        v-if="github.issueCount.value > 0"
        class="
          absolute -top-0.5 -right-0.5
          min-w-[14px] h-[14px] px-0.5
          flex items-center justify-center
          text-[8px] font-bold
          bg-[var(--holo-cyan)] text-black
          rounded-full
        "
      >{{ github.issueCount.value > 999 ? '999+' : github.issueCount.value }}</span>
    </button>

    <!-- My Pull Requests -->
    <button
      ref="prsTriggerRef"
      type="button"
      @click="openPRs"
      class="relative p-1.5 rounded-lg transition-all duration-200 hover:bg-[var(--widget-glass-hover)] cursor-pointer group"
      title="Pull Requests"
    >
      <Icon
        icon="octicon:git-pull-request-16"
        class="w-4 h-4 transition-colors duration-200"
        :class="prColor"
      />
      <!-- Badge -->
      <span
        v-if="github.prCount.value > 0"
        class="
          absolute -top-0.5 -right-0.5
          min-w-[14px] h-[14px] px-0.5
          flex items-center justify-center
          text-[8px] font-bold
          bg-[var(--holo-purple)] text-white
          rounded-full
        "
      >{{ github.prCount.value > 999 ? '999+' : github.prCount.value }}</span>
    </button>

    <!-- Unread Notifications -->
    <button
      ref="notificationsTriggerRef"
      type="button"
      @click="openNotifications"
      class="relative p-1.5 rounded-lg transition-all duration-200 hover:bg-[var(--widget-glass-hover)] cursor-pointer group"
      title="Notifications"
    >
      <Icon
        icon="octicon:inbox-16"
        class="w-4 h-4 transition-colors duration-200"
        :class="notificationColor"
      />
      <!-- Badge -->
      <span
        v-if="github.notificationCount.value > 0"
        class="
          absolute -top-0.5 -right-0.5
          min-w-[14px] h-[14px] px-0.5
          flex items-center justify-center
          text-[8px] font-bold
          bg-[var(--holo-yellow)] text-black
          rounded-full
        "
      >{{ github.notificationCount.value > 999 ? '999+' : github.notificationCount.value }}</span>
    </button>
  </nav>
</template>
