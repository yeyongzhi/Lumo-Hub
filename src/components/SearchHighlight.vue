<script setup lang="ts">
import { computed } from 'vue'
const props = defineProps<{ text: string; query: string }>()
const parts = computed(() => {
  const query = props.query.trim()
  if (!query) return [{ text: props.text, match: false }]
  const pieces: { text: string; match: boolean }[] = []
  const lower = props.text.toLocaleLowerCase(), needle = query.toLocaleLowerCase()
  let start = 0, index = lower.indexOf(needle)
  while (index >= 0) {
    pieces.push({ text: props.text.slice(start, index), match: false }, { text: props.text.slice(index, index + query.length), match: true })
    start = index + query.length; index = lower.indexOf(needle, start)
  }
  pieces.push({ text: props.text.slice(start), match: false })
  return pieces
})
</script>
<template><template v-for="(part,index) in parts" :key="index"><mark v-if="part.match" class="rounded-sm bg-primary/15 text-inherit">{{ part.text }}</mark><template v-else>{{ part.text }}</template></template></template>
