<script setup lang="ts">
import { Window, useCoordinator, Popover, usePopoverMode } from '@arcana/vue'

// Bar components
import HorizontalBar from './components/bars/HorizontalBar.vue'
import VerticalBar from './components/bars/VerticalBar.vue'

// Popover components
import GitHubIssuesPopover from './components/popovers/GitHubIssuesPopover.vue'
import GitHubPRsPopover from './components/popovers/GitHubPRsPopover.vue'
import GitHubNotificationsPopover from './components/popovers/GitHubNotificationsPopover.vue'

const { isPopover } = usePopoverMode()

// Coordinator mode: auto-hide the main window after windows are created
// Don't auto-hide if this is a popover window (popover needs to stay visible)
useCoordinator({ autoHide: !isPopover.value })
</script>

<template>
  <!-- Horizontal bar (top) -->
  <Window
    id="horizontal-bar"
    :position="{ top: 9, left: 55, right: 20, height: 60 }"
  >
    <HorizontalBar />
  </Window>

  <!-- Vertical bar (left) -->
  <Window
    id="vertical-bar"
    :position="{ top: 9, left: 8, bottom: 20, width: 60 }"
  >
    <VerticalBar />
  </Window>

  <!-- GitHub Popovers -->
  <Popover id="github-issues">
    <GitHubIssuesPopover />
  </Popover>
  <Popover id="github-prs">
    <GitHubPRsPopover />
  </Popover>
  <Popover id="github-notifications">
    <GitHubNotificationsPopover />
  </Popover>
</template>

