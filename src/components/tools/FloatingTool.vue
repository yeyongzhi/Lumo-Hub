<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Textarea } from '@/components/ui/textarea';
import { readLocal, writeLocal } from '@/lib/local-files';
import { useToastMessage } from '@/lib/toast-message';
const text = ref(''),
  error = useToastMessage(),
  ready = ref(false);
let timer: ReturnType<typeof setTimeout> | undefined, removeClose: (() => void) | undefined;
let queue = Promise.resolve();
let editVersion = 0,
  disposed = false,
  closing = ref(false);
function save() {
  const version = editVersion,
    value = { text: text.value };
  queue = queue
    .catch(() => {})
    .then(() => writeLocal('lumo-tool-floating', value))
    .then(
      () => {
        if (!disposed && version === editVersion) error.value = '';
      },
      (e) => {
        if (!disposed && version === editVersion) error.value = '保存失败：' + String(e);
        throw e;
      },
    );
  return queue;
}
watch(
  text,
  () => {
    if (!ready.value) return;
    editVersion++;
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => void save().catch(() => {}), 400);
  },
  { flush: 'sync' },
);
onMounted(async () => {
  try {
    const value = await readLocal<{ text: string }>('lumo-tool-floating');
    if (disposed) return;
    if (value) text.value = value.text;
    const unlisten = await getCurrentWindow().onCloseRequested(async (event) => {
      event.preventDefault();
      if (closing.value || !ready.value) return;
      closing.value = true;
      if (timer) clearTimeout(timer);
      try {
        await save();
        await getCurrentWindow().destroy();
      } catch {
        closing.value = false;
      }
    });
    if (disposed) {
      unlisten();
      return;
    }
    removeClose = unlisten;
    ready.value = true;
  } catch (e) {
    if (!disposed) error.value = '初始化失败：' + String(e);
  }
});
onUnmounted(() => {
  disposed = true;
  if (timer) clearTimeout(timer);
  removeClose?.();
});
</script>
<template>
  <div class="flex min-h-0 flex-1 flex-col gap-3 p-5">
    <Textarea
      v-model="text"
      :disabled="!ready || closing"
      placeholder="写下需要一直看见的内容…"
      class="field-sizing-fixed min-h-0 flex-1 resize-none border-0 bg-transparent leading-relaxed shadow-none focus-visible:ring-0 dark:bg-transparent"
      aria-label="悬浮文本"
    />
  </div>
</template>
