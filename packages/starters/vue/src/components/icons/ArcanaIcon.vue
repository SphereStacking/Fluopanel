<script setup lang="ts">
import ArcanaIconSvg from '@/assets/icons/arcana.svg?component'

interface Props {
  class?: string
}
defineProps<Props>()
</script>

<template>
  <ArcanaIconSvg :class="$props.class" />
</template>
