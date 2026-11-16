<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { Button } from '@/components/ui/button';
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue';
import { useWorkspaceStore } from '@/stores/workspace';
type Clip = { id: string; kind: string; content: string };
const entries = ref<Clip[]>([]);
const paused = ref(false);
const total = ref(0);
const error = ref(false);
const store = useWorkspaceStore();
let unlisten: UnlistenFn | undefined;
let disposed = false;
async function refresh() {
  try {
    const data = await invoke<{ entries: Clip[]; paused: boolean }>('clipboard_history');
    if (!disposed) {
      total.value = data.entries.length;
      entries.value = data.entries.filter((item) => item.kind === 'text').slice(0, 5);
      paused.value = data.paused;
      error.value = false;
    }
  } catch {
    if (!disposed) error.value = true;
  }
}
async function copy(id: string) {
  try {
    await invoke('clipboard_copy', { id });
    store.notify('已复制');
  } catch (e) {
    store.notify(String(e));
  }
}
async function clear() {
  try {
    await invoke('clipboard_delete', { id: null });
    await refresh();
  } catch (e) {
    store.notify(String(e));
  }
}
onMounted(async () => {
  try {
    const stop = await listen('lumo-clipboard-changed', refresh);
    if (disposed) stop();
    else unlisten = stop;
  } catch {
    error.value = true;
  }
  await refresh();
});
onUnmounted(() => {
  disposed = true;
  unlisten?.();
});
</script>
<template>
  <div class="flex min-h-0 flex-1 flex-col gap-1">
    <p v-if="paused" class="text-xs text-muted-foreground">已暂停记录</p>
    <div v-if="entries.length" class="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto">
      <Button
        v-for="entry in entries"
        :key="entry.id"
        variant="ghost"
        class="h-8 w-full justify-start px-2 text-left"
        :title="entry.content"
        @click="copy(entry.id)"
        ><span class="truncate">{{ entry.content.replace(/\s+/g, ' ') }}</span></Button
      >
    </div>
    <p v-else class="flex flex-1 items-center justify-center text-sm text-muted-foreground">
      {{ error ? '无法读取剪贴板历史' : '暂无文本记录' }}
    </p>
    <ConfirmDialog
      v-if="total"
      title="清空剪贴板历史？"
      description="将删除全部文本和图片历史，系统剪贴板内容不受影响。"
      confirm-text="清空历史"
      destructive
      @confirm="clear"
      ><template #trigger
        ><Button variant="danger" size="sm" class="self-end">清空历史</Button></template
      ></ConfirmDialog
    >
  </div>
</template>
