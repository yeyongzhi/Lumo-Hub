<script setup lang="ts">
import { ref } from 'vue'
import { api } from '@/lib/api'
import { useWorkspaceStore } from '@/stores/workspace'
import type { Countdown, SearchResult } from '@/lib/types'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
const emit=defineEmits<{saved:[item:SearchResult]}>()
const props=withDefaults(defineProps<{initialText?:string; initialKind?:string}>(),{initialText:'',initialKind:'sticky'})
const savedCountdown=ref<Countdown|null>(null)
const store=useWorkspaceStore(), kind=ref(props.initialKind), text=ref(props.initialText), date=ref(''), busy=ref(false), savedId=ref(''), error=ref('')
async function save(){
  if(busy.value||!text.value.trim()||(kind.value==='countdown'&&!date.value))return
  busy.value=true;error.value=''
  try{
    await store.flushPendingSaves()
    const title=text.value.trim().split('\n')[0]!.slice(0,120)
    if(kind.value==='countdown'){
      const value={title,description:text.value.trim(),target_date:date.value,repeat_type:'once' as const};if(savedCountdown.value){await api.updateCountdown({...savedCountdown.value,...value})}else{savedCountdown.value=await api.createCountdown(value);savedId.value=savedCountdown.value.id}
    }else if(kind.value==='note'){
      if(!savedId.value)savedId.value=(await api.createNote(title)).id
      await api.updateNote(savedId.value,title,text.value.trim())
    }else{
      if(!savedId.value)savedId.value=(await api.createSticky(title)).id
      await api.updateSticky(savedId.value,title,text.value.trim())
    }
    const result={id:savedId.value,title,subtitle:text.value.trim(),kind:kind.value as SearchResult['kind']}
    await store.reload();text.value='';savedId.value='';savedCountdown.value=null;store.notify('已保存');emit('saved',result)
  }catch(e){error.value=`保存失败：${String(e)}。内容已保留，请重试。`}finally{busy.value=false}
}
</script>
<template><form class="flex flex-col gap-3" @submit.prevent="save"><FieldGroup><Field><FieldLabel class="sr-only">记录类型</FieldLabel><Select v-model="kind" :disabled="busy||!!savedId"><SelectTrigger aria-label="记录类型" class="w-32"><SelectValue/></SelectTrigger><SelectContent><SelectGroup><SelectItem value="sticky">便签</SelectItem><SelectItem value="note">笔记</SelectItem><SelectItem value="countdown">重要日期</SelectItem></SelectGroup></SelectContent></Select></Field><Field><FieldLabel class="sr-only" for="quick-capture-text">记录内容</FieldLabel><Textarea id="quick-capture-text" v-model="text" :disabled="busy" placeholder="记下一句话、一个想法，或重要日期…" class="min-h-20" @keydown.ctrl.enter.prevent="save"/></Field><Field v-if="kind==='countdown'"><FieldLabel for="quick-capture-date">日期</FieldLabel><Input id="quick-capture-date" v-model="date" type="date" :disabled="busy"/></Field></FieldGroup><div class="flex items-center justify-between gap-2"><span class="text-xs text-muted-foreground">Ctrl + Enter 保存</span><Button type="submit" :disabled="busy||!text.trim()||(kind==='countdown'&&!date)">{{ busy?'保存中…':'保存记录' }}</Button></div><p v-if="error" class="text-sm text-destructive" role="alert">{{ error }}</p></form></template>
