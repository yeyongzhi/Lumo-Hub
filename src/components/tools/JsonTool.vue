<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Field, FieldLabel } from '@/components/ui/field';
import { formatJson, jsonError } from '@/lib/json-tool';

const source = ref('');
const output = ref('');
const message = ref('');
const invalid = ref(false);

function clearResult() {
  output.value = '';
  message.value = '';
  invalid.value = false;
}

function run(action: 'format' | 'compact' | 'validate') {
  clearResult();
  try {
    if (action === 'validate') JSON.parse(source.value);
    else output.value = formatJson(source.value, action === 'compact');
    message.value = action === 'validate' ? 'JSON 语法正确。' : '转换完成。';
  } catch (error) {
    const issue = jsonError(source.value, error);
    invalid.value = true;
    message.value = issue.message;
    if (issue.offset !== null) {
      const editor = document.querySelector<HTMLTextAreaElement>('#json-source');
      editor?.focus();
      editor?.setSelectionRange(issue.offset, issue.offset + 1);
    }
  }
}

async function copy() {
  try {
    await invoke('clipboard_copy', { text: output.value });
    message.value = '已复制结果。';
  } catch (error) {
    message.value = '复制失败：' + String(error);
  }
}
</script>

<template>
  <div class="flex flex-col gap-4">
    <div class="grid grid-cols-2 gap-4">
      <Field class="min-w-0" :data-invalid="invalid || undefined">
        <FieldLabel for="json-source">输入 JSON</FieldLabel>
        <Textarea
          id="json-source"
          v-model="source"
          class="field-sizing-fixed h-80 resize-none font-mono"
          placeholder="粘贴 JSON 文本…"
          spellcheck="false"
          :aria-invalid="invalid"
          :aria-describedby="message ? 'json-status' : undefined"
          @update:model-value="clearResult"
        />
      </Field>
      <Field class="min-w-0">
        <FieldLabel for="json-output">转换结果</FieldLabel>
        <Textarea
          id="json-output"
          :model-value="output"
          readonly
          spellcheck="false"
          class="field-sizing-fixed h-80 resize-none font-mono"
        />
      </Field>
    </div>
    <div class="flex flex-wrap gap-2">
      <Button @click="run('format')">格式化</Button>
      <Button variant="outline" @click="run('compact')">压缩</Button>
      <Button variant="outline" @click="run('validate')">校验</Button>
      <Button variant="outline" :disabled="!output" @click="copy">复制结果</Button>
    </div>
    <p
      v-if="message"
      id="json-status"
      role="status"
      class="text-sm wrap-anywhere"
      :class="invalid ? 'text-destructive' : 'text-muted-foreground'"
    >
      {{ message }}
    </p>
  </div>
</template>
