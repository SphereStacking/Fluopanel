<script setup lang="ts">
import { executeShell } from '@arcana/core'
import { Icon } from '@iconify/vue'

const menuItems = [
  { label: 'Arcana 設定', icon: 'mdi:cog', command: 'code ~/.config/arcana/config.json' },
  { label: 'Aerospace 設定', icon: 'mdi:window-restore', command: 'code ~/.config/aerospace/aerospace.toml' },
  { label: '.zshrc', icon: 'mdi:console', command: 'code ~/.zshrc' },
  { label: '.claude/', icon: 'mdi:folder-cog', command: 'code ~/.claude' },
]

const handleClick = async (command: string) => {
  await executeShell(command)
}
</script>

<template>
  <div
    class="relative font-[-apple-system,BlinkMacSystemFont,'SF_Pro_Text',sans-serif] text-[13px] antialiased"
  >
    <!-- Glass background -->
    <div
      class="absolute inset-0 rounded-xl border border-glass-border bg-glass-bg backdrop-blur-[40px] backdrop-saturate-[180%] shadow-[0_0_0_0.5px_rgba(0,0,0,0.3),0_24px_48px_-12px_rgba(0,0,0,0.5),0_12px_24px_-8px_rgba(0,0,0,0.3),inset_0_1px_0_rgba(255,255,255,0.05)]"
    />

    <div class="relative flex flex-col rounded-xl overflow-hidden">
      <!-- Header -->
      <header class="flex items-center justify-between px-3.5 py-2.5 border-b border-white/[0.04]">
        <div class="flex items-center gap-2">
          <Icon icon="mdi:cog" class="w-4 h-4 text-holo-cyan" />
          <span class="text-text-primary font-semibold tracking-tight">Settings</span>
        </div>
      </header>

      <!-- Menu Items -->
      <div class="flex-1 py-1">
        <div
          v-for="(item, index) in menuItems"
          :key="item.label"
          @click="handleClick(item.command)"
          class="group flex items-center gap-2.5 mx-1.5 px-2.5 py-2 rounded-lg cursor-pointer transition-all duration-100 hover:bg-widget-glass-hover active:bg-widget-glass active:scale-[0.98] animate-fade-slide-in"
          :style="{ animationDelay: `${index * 30}ms` }"
        >
          <!-- Icon -->
          <Icon
            :icon="item.icon"
            class="w-4 h-4 text-text-tertiary transition-colors duration-100 group-hover:text-holo-cyan"
          />

          <!-- Label -->
          <span
            class="flex-1 text-[12px] text-text-primary transition-colors duration-100 group-hover:text-white"
          >
            {{ item.label }}
          </span>

          <!-- Arrow -->
          <Icon
            icon="mdi:chevron-right"
            class="w-3 h-3 text-text-ghost opacity-0 -translate-x-1 transition-all duration-100 group-hover:opacity-60 group-hover:translate-x-0"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
@keyframes fade-slide-in {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-fade-slide-in {
  animation: fade-slide-in 0.2s ease-out forwards;
  opacity: 0;
}
</style>
