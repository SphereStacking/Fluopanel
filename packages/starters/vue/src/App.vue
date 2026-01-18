<script setup lang="ts">
import { createProviders } from '@arcana/providers'
import { Widget, useCoordinator, Popup, usePopupMode } from '@arcana/vue'
import Bar from './components/Bar.vue'
import GitHubIssuesPopup from './components/popups/GitHubIssuesPopup.vue'
import GitHubPRsPopup from './components/popups/GitHubPRsPopup.vue'
import GitHubNotificationsPopup from './components/popups/GitHubNotificationsPopup.vue'

const providers = createProviders()
const { isPopup } = usePopupMode()

// Coordinator mode: auto-hide the main window after widgets are created
// Don't auto-hide if this is a popup window (popup needs to stay visible)
useCoordinator({ autoHide: !isPopup.value })
</script>

<template>
  <!-- Main bar widget (only created in coordinator mode) -->
  <Widget
    id="bar"
    :position="{ top: 9, left: 20, right: 20, height: 60 }"
  >
    <Bar :providers="providers" />
  </Widget>

  <!-- GitHub Popups -->
  <Popup id="github-issues">
    <GitHubIssuesPopup />
  </Popup>
  <Popup id="github-prs">
    <GitHubPRsPopup />
  </Popup>
  <Popup id="github-notifications">
    <GitHubNotificationsPopup />
  </Popup>
</template>
