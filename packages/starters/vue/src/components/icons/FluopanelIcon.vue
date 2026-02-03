<script setup lang="ts">
import FluopanelIconSvg from '@/assets/icons/fluopanel.svg?component'

interface Props {
  class?: string
}
defineProps<Props>()
</script>

<template>
  <FluopanelIconSvg :class="$props.class" />
</template>
