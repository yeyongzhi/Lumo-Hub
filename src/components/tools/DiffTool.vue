<script setup lang="ts">
import { computed, ref } from 'vue';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Field, FieldLabel } from '@/components/ui/field';
import { ScrollArea } from '@/components/ui/scroll-area';
import { compareLines, type DiffRow } from '@/lib/line-diff';

const before = ref('');
const after = ref('');
const rows = ref<DiffRow[]>([]);
const message = ref('粘贴两份文本后开始对比。');
const styles = {
  same: '',
  added: 'bg-emerald-500/15',
  removed: 'bg-destructive/15',
  changed: 'bg-amber-500/20',
};
const marks = { same: ' ', added: '+', removed: '−', changed: '~' };
const summary = computed(() => {
  const counts = { added: 0, removed: 0, changed: 0, same: 0 };
  for (const row of rows.value) counts[row.kind]++;
  return '新增 ' + counts.added + ' · 删除 ' + counts.removed + ' · 修改 ' + counts.changed;
});
const revision = ref(0);
function compare() {
  try {
    rows.value = compareLines(before.value, after.value);
    message.value = rows.value.some((row) => row.kind !== 'same')
      ? summary.value
      : '两份文本内容一致。';
    revision.value++;
  } catch (error) {
    rows.value = [];
    message.value = error instanceof Error ? error.message : String(error);
  }
}
function invalidate() {
  rows.value = [];
  message.value = '文本已编辑，请重新对比。';
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col gap-4 p-5">
    <div class="grid shrink-0 grid-cols-2 gap-4">
      <Field class="min-w-0">
        <FieldLabel for="diff-before">原文</FieldLabel>
        <Textarea
          id="diff-before"
          v-model="before"
          class="field-sizing-fixed h-40 resize-none font-mono"
          spellcheck="false"
          @update:model-value="invalidate"
        />
      </Field>
      <Field class="min-w-0">
        <FieldLabel for="diff-after">修改稿</FieldLabel>
        <Textarea
          id="diff-after"
          v-model="after"
          class="field-sizing-fixed h-40 resize-none font-mono"
          spellcheck="false"
          @update:model-value="invalidate"
        />
      </Field>
    </div>
    <div class="flex shrink-0 flex-wrap items-center gap-4">
      <Button @click="compare">对比文本</Button>
      <p role="status" class="text-sm text-muted-foreground">{{ message }}</p>
    </div>
    <div v-if="rows.length" class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border">
      <div class="grid shrink-0 grid-cols-2 divide-x border-b bg-muted text-sm font-medium">
        <span class="px-3 py-2">原文</span>
        <span class="px-3 py-2">修改稿</span>
      </div>
      <ScrollArea :key="revision" class="min-h-0 flex-1">
        <div
          v-for="(row, index) in rows"
          :key="index"
          class="grid grid-cols-2 divide-x border-b last:border-b-0"
          :class="styles[row.kind]"
        >
          <div
            v-for="side in ['before', 'after'] as const"
            :key="side"
            class="flex min-w-0 gap-2 px-2 py-1 font-mono text-sm"
          >
            <span class="w-12 shrink-0 text-right text-muted-foreground"
              >{{ row[side]?.number }} {{ row[side] ? marks[row.kind] : '' }}</span
            >
            <span class="min-w-0 wrap-anywhere whitespace-pre-wrap">{{ row[side]?.text }}</span>
          </div>
        </div>
      </ScrollArea>
    </div>
  </div>
</template>
