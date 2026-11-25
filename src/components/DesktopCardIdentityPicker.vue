<script setup lang="ts">
import { ref, watch } from 'vue';
import { desktopCardEmojiOptions, desktopCardIconOptions } from '@/lib/desktop-card-icons';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

const props = withDefaults(
  defineProps<{
    icon: string | null;
    emoji: string | null;
    initialTab?: string;
  }>(),
  { initialTab: 'icon' },
);
const activeTab = ref(props.initialTab);
watch(
  () => props.initialTab,
  (value) => (activeTab.value = value),
);
const emit = defineEmits<{
  'update:icon': [value: string | null];
  'update:emoji': [value: string | null];
}>();

function chooseIcon(value: string | null) {
  emit('update:icon', value);
  emit('update:emoji', null);
}
function chooseEmoji(value: string) {
  emit('update:icon', null);
  emit('update:emoji', value);
}
</script>

<template>
  <Tabs v-model="activeTab" class="w-full">
    <TabsList class="w-full">
      <TabsTrigger value="icon">卡片标识</TabsTrigger>
      <TabsTrigger value="emoji">Emoji</TabsTrigger>
    </TabsList>
    <TabsContent value="icon">
      <div class="grid grid-cols-5 gap-2">
        <button
          v-for="option in desktopCardIconOptions"
          :key="option.value"
          type="button"
          class="flex flex-col items-center gap-1 rounded-lg border p-2 text-xs transition-colors hover:bg-accent"
          :class="icon === option.value ? 'border-primary bg-primary/10' : 'border-border'"
          :aria-label="option.label"
          :aria-pressed="icon === option.value"
          :title="option.label"
          @click="chooseIcon(option.value)"
        >
          <component :is="option.component" class="size-5" />
          {{ option.label }}
        </button>
      </div>
    </TabsContent>
    <TabsContent value="emoji">
      <div class="grid grid-cols-10 gap-1.5">
        <button
          v-for="option in desktopCardEmojiOptions"
          :key="option"
          type="button"
          class="flex size-9 items-center justify-center rounded-lg border text-lg transition-colors hover:bg-accent"
          :class="emoji === option ? 'border-primary bg-primary/10' : 'border-border'"
          :aria-label="`选择 Emoji ${option}`"
          :aria-pressed="emoji === option"
          @click="chooseEmoji(option)"
        >
          {{ option }}
        </button>
      </div>
    </TabsContent>
  </Tabs>
</template>
