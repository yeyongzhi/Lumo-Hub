<script setup lang="ts">
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field'
import { textOperations, transformText } from '@/lib/text-tools'
const input=ref(''),output=ref(''),message=ref('')
const count=computed(()=>Array.from(input.value).length)
function run(id:string){try{output.value=transformText(id,input.value);message.value='处理完成'}catch(e){message.value='处理失败，请检查输入格式：'+String(e)}}
async function copy(){try{await invoke('clipboard_copy',{text:output.value});message.value='已复制结果'}catch(e){message.value=String(e)}}
</script>
<template>
 <div class="flex flex-col gap-4"><FieldGroup><Field><FieldLabel for="text-source">输入 · {{count}} 字符 · {{input?input.split(/\r?\n/).length:0}} 行</FieldLabel><Textarea id="text-source" v-model="input" class="min-h-40 font-mono" placeholder="粘贴需要处理的文本…"/></Field></FieldGroup>
 <div class="flex flex-wrap gap-2"><Button v-for="operation in textOperations" :key="operation.id" variant="outline" size="sm" @click="run(operation.id)">{{operation.label}}</Button></div>
 <FieldGroup><Field><FieldLabel for="text-result">处理结果</FieldLabel><Textarea id="text-result" :model-value="output" readonly class="min-h-40 font-mono"/></Field></FieldGroup><div class="flex gap-2"><Button :disabled="!output" @click="copy">复制结果</Button><Button variant="outline" :disabled="!output" @click="input=output">结果作为输入</Button><Button variant="ghost" @click="input='';output='';message=''">清空</Button></div><p v-if="message" role="status" class="text-sm text-muted-foreground">{{message}}</p></div>
</template>
