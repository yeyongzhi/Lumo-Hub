<script setup lang="ts">
import { computed } from 'vue';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import type { Note, Sticky } from '@/lib/types';

const props = defineProps<{ record: Note | Sticky | null; kind: '笔记' | '便签' }>();
const emit = defineEmits<{ close: [] }>();
const characters = computed(() => Array.from(props.record?.content ?? '').length);
function date(value: string) {
  const parsed = new Date(value);
  return Number.isNaN(parsed.getTime()) ? '未知' : parsed.toLocaleString('zh-CN');
}
</script>

<template>
  <Dialog
    :open="!!record"
    @update:open="
      (open) => {
        if (!open) emit('close');
      }
    "
  >
    <DialogContent>
      <DialogHeader>
        <DialogTitle>{{ kind }}详情</DialogTitle>
        <DialogDescription>查看{{ kind }}的基本信息。</DialogDescription>
      </DialogHeader>
      <dl v-if="record" class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-6 gap-y-3 text-sm">
        <dt class="text-muted-foreground">标题</dt>
        <dd class="break-words">{{ record.title || '无标题' }}</dd>
        <dt class="text-muted-foreground">创建日期</dt>
        <dd>{{ date(record.created_at) }}</dd>
        <dt class="text-muted-foreground">最近更新</dt>
        <dd>{{ date(record.updated_at) }}</dd>
        <dt class="text-muted-foreground">正文总字数</dt>
        <dd>{{ characters }} 字</dd>
      </dl>
    </DialogContent>
  </Dialog>
</template>
