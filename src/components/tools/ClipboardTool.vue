<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Copy, Trash2 } from '@lucide/vue';
import { Button } from '@/components/ui/button';
import { ButtonGroup } from '@/components/ui/button-group';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue';
import { useToastMessage } from '@/lib/toast-message';
type Clip = { id: string; kind: string; content: string; created: string };
const emit = defineEmits<{ 'count-change': [count: number] }>();
const entries = ref<Clip[]>([]),
  paused = ref(false),
  query = ref(''),
  filter = ref('all'),
  message = useToastMessage();
const visible = computed(() =>
  entries.value.filter(
    (e) =>
      (filter.value === 'all' || e.kind === filter.value) &&
      (e.kind === 'image'
        ? !query.value
        : e.content.toLowerCase().includes(query.value.toLowerCase())),
  ),
);
let unlisten: (() => void) | undefined;
async function refresh() {
  try {
    const data = await invoke<{ entries: Clip[]; paused: boolean }>('clipboard_history');
    entries.value = data.entries;
    emit('count-change', data.entries.length);
    paused.value = data.paused;
  } catch (e) {
    message.value = String(e);
  }
}
async function pause(value: boolean) {
  try {
    await invoke('clipboard_pause', { paused: value });
    paused.value = value;
  } catch (e) {
    message.value = String(e);
  }
}
async function remove(id: string | null) {
  try {
    await invoke('clipboard_delete', { id });
    await refresh();
  } catch (e) {
    message.value = String(e);
  }
}
async function copy(id: string) {
  try {
    await invoke('clipboard_copy', { id });
    message.value = '已复制，可切换到目标应用粘贴';
  } catch (e) {
    message.value = String(e);
  }
}
onMounted(async () => {
  unlisten = await listen('lumo-clipboard-changed', refresh);
  await refresh();
});
onUnmounted(() => unlisten?.());
</script>
<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <div class="flex shrink-0 flex-col gap-4 border-b px-5 py-4">
      <p class="text-sm text-muted-foreground">
        应用运行期间自动记录文本和图片，仅保存在本机。最多 200 条 / 32 MB；单条文本 1
        MB、图片原始像素 16 MB。
      </p>
      <div class="flex flex-wrap items-center gap-3">
        <Tabs v-model="filter"
          ><TabsList
            ><TabsTrigger value="all">全部</TabsTrigger><TabsTrigger value="text">文本</TabsTrigger
            ><TabsTrigger value="image">图片</TabsTrigger></TabsList
          ></Tabs
        >
        <Input
          v-model="query"
          class="min-w-40 flex-1"
          placeholder="搜索文本历史…"
          aria-label="搜索剪贴板"
        /><ButtonGroup aria-label="剪贴板记录操作">
          <Button variant="outline" :aria-pressed="paused" @click="pause(!paused)">{{
            paused ? '恢复记录' : '暂停记录'
          }}</Button
          ><ConfirmDialog
            title="清空剪贴板历史？"
            description="删除本工具保存的全部记录，系统剪贴板内容不受影响。"
            destructive
            @confirm="remove(null)"
            ><template #trigger
              ><Button variant="outline" :disabled="!entries.length">清空历史</Button></template
            ></ConfirmDialog
          ></ButtonGroup
        >
      </div>
    </div>
    <ScrollArea class="min-h-0 flex-1">
      <div class="flex flex-col gap-4 p-5">
        <p v-if="!visible.length" class="py-8 text-center text-sm text-muted-foreground">
          暂无记录，复制一段文本或一张图片试试。
        </p>
        <article
          v-for="entry in visible"
          :key="entry.id"
          class="flex flex-col gap-3 rounded-xl border bg-card p-4"
        >
          <div class="flex items-center gap-2">
            <span class="flex-1 text-xs text-muted-foreground"
              >{{ entry.kind === 'image' ? '图片' : '文本' }} ·
              {{ new Date(entry.created).toLocaleString() }}</span
            ><Button size="icon-sm" variant="ghost" aria-label="复制" @click="copy(entry.id)"
              ><Copy /></Button
            ><Button size="icon-sm" variant="danger" aria-label="删除" @click="remove(entry.id)"
              ><Trash2
            /></Button>
          </div>
          <img
            v-if="entry.kind === 'image'"
            :src="entry.content"
            alt="剪贴板图片"
            class="max-h-72 max-w-full self-start rounded-lg object-contain"
          />
          <pre v-else class="max-h-48 overflow-hidden whitespace-pre-wrap break-all text-sm">{{
            entry.content
          }}</pre>
        </article>
      </div>
    </ScrollArea>
  </div>
</template>
