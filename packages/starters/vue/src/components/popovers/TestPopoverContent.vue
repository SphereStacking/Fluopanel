<script setup lang="ts">
import { ref } from 'vue'

const itemCount = ref(5)
const items = [5, 10, 20, 50, 100]
</script>

<template>
  <!-- Root container fills parent height for proper scroll behavior -->
  <div class="relative h-full font-[-apple-system,BlinkMacSystemFont,'SF_Pro_Text',sans-serif] text-[13px] antialiased">
    <!-- Glass background -->
    <div
      class="absolute inset-0 rounded-xl border border-glass-border bg-glass-bg backdrop-blur-[40px] backdrop-saturate-[180%]"
    />

    <div class="relative flex flex-col h-full rounded-xl overflow-hidden min-w-[320px]">
      <!-- Header (fixed) -->
      <header class="flex-shrink-0 flex items-center justify-between px-4 py-3 border-b border-white/[0.04]">
        <span class="text-text-primary font-semibold">Max Size Test</span>
        <span class="text-[10px] text-text-tertiary">{{ itemCount }} items</span>
      </header>

      <!-- Item count selector (fixed) -->
      <div class="flex-shrink-0 flex flex-wrap gap-1 px-3 py-2 border-b border-white/[0.04]">
        <button
          v-for="count in items"
          :key="count"
          @click="itemCount = count"
          class="px-3 py-1 text-[11px] rounded transition-colors"
          :class="itemCount === count
            ? 'bg-holo-cyan/20 text-holo-cyan'
            : 'bg-widget-glass text-text-secondary hover:bg-widget-glass-hover'"
        >
          {{ count }}
        </button>
      </div>

      <!-- Scrollable content area -->
      <div class="flex-1 min-h-0 overflow-y-auto popup-scrollbar">
        <div class="py-1">
          <div
            v-for="i in itemCount"
            :key="i"
            class="flex items-center gap-2 mx-1.5 px-2 py-2 rounded-lg hover:bg-widget-glass-hover transition-colors"
          >
            <span class="w-6 h-6 rounded-full bg-holo-purple/20 flex items-center justify-center text-[10px] text-holo-purple font-bold">
              {{ i }}
            </span>
            <span class="text-[12px] text-text-primary">Test Item {{ i }}</span>
          </div>
        </div>
      </div>

      <!-- Footer (fixed) -->
      <footer class="flex-shrink-0 px-3 py-2 border-t border-white/[0.04]">
        <div class="text-[10px] text-text-tertiary text-center">
          Screen size limit test - scroll to see more
        </div>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.popup-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.popup-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.popup-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 3px;
}
.popup-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.25);
}
</style>
