<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { createBatteryProvider } from '@arcana/providers'

// Reactive state
const battery = ref(null)
const loading = ref(true)
const error = ref(null)

// Provider instance
const provider = createBatteryProvider()
let unsubscribe = null

onMounted(async () => {
  try {
    // Initial fetch
    battery.value = await provider.get()
    loading.value = false

    // Subscribe to updates
    unsubscribe = provider.subscribe((data) => {
      battery.value = data
    })
  } catch (e) {
    error.value = e.message
    loading.value = false
  }
})

onUnmounted(() => {
  // Cleanup subscription
  unsubscribe?.()
})
</script>

<template>
  <div class="widget">
    <h1>My Widget</h1>

    <div v-if="loading" class="loading">
      Loading...
    </div>

    <div v-else-if="error" class="error">
      {{ error }}
    </div>

    <div v-else class="content">
      <div class="stat">
        <span class="label">Battery</span>
        <span class="value">{{ battery?.percentage }}%</span>
      </div>
      <div class="status">
        {{ battery?.isCharging ? 'Charging' : 'On Battery' }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.widget {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: rgba(30, 30, 30, 0.9);
  color: white;
  padding: 16px;
  border-radius: 12px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

h1 {
  font-size: 14px;
  font-weight: 500;
  margin: 0 0 12px 0;
  opacity: 0.7;
}

.loading, .error {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.error {
  color: #f87171;
}

.content {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.stat {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 8px;
}

.label {
  font-size: 14px;
  opacity: 0.8;
}

.value {
  font-size: 32px;
  font-weight: 600;
  color: #4ade80;
}

.status {
  font-size: 12px;
  opacity: 0.6;
}
</style>
