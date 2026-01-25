<script setup lang="ts">
import { ref } from 'vue'
import { Icon } from '@iconify/vue'
import { usePopover } from '@arcana/vue'

const triggerRef = ref<HTMLElement | null>(null)

const props = defineProps<{
  popoverId: string
  // Define any props if needed
}>()

const popover = usePopover({
  width: 300,
  height: 200,
  align: 'center',
  offsetY: 8,
  exclusive: true,
})

async function openPopover() {
  if (triggerRef.value) {
    await popover.toggle(props.popoverId , triggerRef.value)
  }
}
</script>

<template>
  <button
    ref="triggerRef"
    type="button"
    @click="openPopover"
    class="flex items-center gap-1 py-1 px-2 rounded-lg text-[12px] transition-all duration-200 hover:bg-widget-glass-hover cursor-pointer"
  >
    <Icon icon="mdi:test-tube" class="w-[14px] h-[14px] text-holo-cyan" />
    <span class="text-text-secondary">Test</span>
  </button>
</template>
