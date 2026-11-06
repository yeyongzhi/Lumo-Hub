<script setup lang="ts">
import { defineAsyncComponent, onMounted, onUnmounted, ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { Pin, Minus, X } from '@lucide/vue';
import { builtinTools } from '@/lib/tools';
import { api } from '@/lib/api';
import { applySettings, watchSystemTheme } from '@/lib/theme';
import type { AppSettings } from '@/lib/types';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
const CalendarTool = defineAsyncComponent(() => import('@/components/tools/CalendarTool.vue'));
const ColorTool = defineAsyncComponent(() => import('@/components/tools/ColorTool.vue'));
const ClipboardTool = defineAsyncComponent(() => import('@/components/tools/ClipboardTool.vue'));
const QrTool = defineAsyncComponent(() => import('@/components/tools/QrTool.vue'));
const TextTool = defineAsyncComponent(() => import('@/components/tools/TextTool.vue'));
const FloatingTool = defineAsyncComponent(() => import('@/components/tools/FloatingTool.vue'));
const TimestampTool = defineAsyncComponent(() => import('@/components/tools/TimestampTool.vue'));
const id = new URLSearchParams(location.search).get('tool') ?? 'color';
const tool = builtinTools.find((t) => t.id === id) ?? builtinTools[0];
const floating = tool.id === 'floating';
const components = {
  calendar: CalendarTool,
  color: ColorTool,
  clipboard: ClipboardTool,
  qrcode: QrTool,
  text: TextTool,
  floating: FloatingTool,
  timestamp: TimestampTool,
  json: defineAsyncComponent(() => import('@/components/tools/JsonTool.vue')),
  diff: defineAsyncComponent(() => import('@/components/tools/DiffTool.vue')),
  timer: defineAsyncComponent(() => import('@/components/tools/TimerTool.vue')),
  units: defineAsyncComponent(() => import('@/components/tools/UnitsTool.vue')),
};
const pinned = ref(id === 'floating'),
  error = ref('');
const removers: (() => void)[] = [];
let settings: AppSettings | undefined;
let settingsVersion = 0;
let disposed = false;
async function pin() {
  try {
    await getCurrentWindow().setAlwaysOnTop(!pinned.value);
    pinned.value = !pinned.value;
  } catch (e) {
    error.value = String(e);
  }
}
function drag(event: MouseEvent) {
  if (event.button === 0 && !(event.target as HTMLElement).closest('button'))
    void getCurrentWindow().startDragging();
}
async function refreshSettings() {
  const version = ++settingsVersion;
  try {
    const next = (await api.bootstrap()).settings;
    if (disposed || version !== settingsVersion) return;
    if (!settings) removers.push(watchSystemTheme(() => settings ?? next));
    settings = next;
    applySettings(settings);
  } catch (e) {
    if (!disposed && version === settingsVersion) error.value = String(e);
  }
}
onMounted(async () => {
  try {
    const unlisten = await listen('lumo-settings-changed', () => void refreshSettings());
    if (disposed) {
      unlisten();
      return;
    }
    removers.push(unlisten);
  } catch (e) {
    if (!disposed) error.value = String(e);
  }
  if (!disposed) await refreshSettings();
});
onUnmounted(() => {
  disposed = true;
  removers.forEach((remove) => remove());
});
</script>
<template>
  <div
    class="tool-window flex h-dvh min-w-0 flex-col overflow-hidden bg-background text-foreground"
    :class="{ border: floating }"
  >
    <header class="flex h-14 shrink-0 items-center gap-3 border-b px-4" @mousedown="drag">
      <component :is="tool.icon" class="size-5 text-primary" />
      <h1 class="min-w-0 flex-1 truncate text-sm font-semibold">
        {{ tool.title }}
      </h1>
      <Button
        variant="ghost"
        size="icon-sm"
        :aria-pressed="pinned"
        :title="pinned ? '取消置顶' : '置顶窗口'"
        aria-label="切换置顶"
        @click="pin"
        ><Pin :class="pinned ? 'text-primary' : ''"
      /></Button>
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="最小化"
        @click="getCurrentWindow().minimize()"
        ><Minus
      /></Button>
      <Button variant="ghost" size="icon-sm" aria-label="关闭" @click="getCurrentWindow().close()"
        ><X
      /></Button>
    </header>
    <p v-if="error" role="alert" class="px-5 pt-3 text-sm text-destructive">{{ error }}</p>
    <component
      :is="components[tool.id]"
      v-if="floating || tool.id === 'clipboard' || tool.id === 'diff'"
    />
    <ScrollArea v-else class="min-h-0 flex-1"
      ><div class="p-5"><component :is="components[tool.id]" /></div
    ></ScrollArea>
  </div>
</template>

<style scoped>
.tool-window :deep([data-slot='scroll-area-scrollbar']) {
  visibility: hidden;
}

.tool-window :deep(textarea),
:global(body:has(.tool-window) [data-slot='select-content']) {
  scrollbar-width: none;
}
</style>
