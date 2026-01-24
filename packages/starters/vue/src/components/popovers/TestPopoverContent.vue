<script setup lang="ts">
</script>

<template>
  <div class="relative w-full h-full font-[-apple-system,BlinkMacSystemFont,'SF_Pro_Text',sans-serif] text-[13px] antialiased">
    <!-- Glass background -->
    <div
      class="absolute inset-0 rounded-xl border border-glass-border bg-glass-bg backdrop-blur-[40px] backdrop-saturate-[180%] shadow-[0_0_0_0.5px_rgba(0,0,0,0.3),0_24px_48px_-12px_rgba(0,0,0,0.5),0_12px_24px_-8px_rgba(0,0,0,0.3),inset_0_1px_0_rgba(255,255,255,0.05)]"
    />

    <div class="relative flex flex-col h-full rounded-xl overflow-hidden p-4">
      <h3 class="text-lg font-bold mb-2 text-text-primary">Test Popover</h3>
      <p class="text-sm text-text-secondary mb-4">
        外部をクリックしてこの Popover が閉じるか確認してください。
      </p>
      <div class="flex gap-2">
        <button class="px-3 py-1 bg-holo-cyan/20 rounded text-holo-cyan text-sm">
          Button 1
        </button>
        <button class="px-3 py-1 bg-holo-purple/20 rounded text-holo-purple text-sm">
          Button 2
        </button>
      </div>
    </div>
  </div>
</template>
