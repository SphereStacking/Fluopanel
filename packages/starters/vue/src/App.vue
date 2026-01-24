<script setup lang="ts">
import { Window, useCoordinator, Popover, usePopoverMode } from '@arcana/vue'
import Bar from './components/Bar.vue'
import GitHubPRsPopover from './components/popovers/GitHubPRsPopover.vue'
import GitHubNotificationsPopover from './components/popovers/GitHubNotificationsPopover.vue'
import TestPopoverContent from './components/popovers/TestPopoverContent.vue'

const { isPopover } = usePopoverMode()

// Coordinator mode: auto-hide the main window after windows are created
// Don't auto-hide if this is a popover window (popover needs to stay visible)
useCoordinator({ autoHide: !isPopover.value })
</script>

<template>
  <!-- Main bar window (only created in coordinator mode) -->
  <Window
    id="bar"
    :position="{ top: 9, left: 20, right: 20, height: 60 }"
  >
    <Bar />
  </Window>

  <!-- GitHub Popovers -->
  <Popover id="github-prs">
    <GitHubPRsPopover />
  </Popover>
  <Popover id="github-notifications">
    <GitHubNotificationsPopover />
  </Popover>

  <!-- Test Popover -->
  <Popover id="test-popover">
    <TestPopoverContent />
  </Popover>
</template>
