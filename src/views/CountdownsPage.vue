<script setup lang="ts">
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { FieldSet, FieldLegend } from '@/components/ui/field'

import { computed, nextTick, reactive, ref, watch } from 'vue'
import { CalendarDays, Pencil, Plus, RotateCcw, Trash2 } from '@lucide/vue'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Field } from '@/components/ui/field'
import { FieldGroup } from '@/components/ui/field'
import { FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Calendar } from '@/components/ui/calendar'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { parseDate, type DateValue } from '@internationalized/date'
import { Textarea } from '@/components/ui/textarea'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { countdownStatus } from '@/lib/dates'
import { api } from '@/lib/api'
import { useWorkspaceStore } from '@/stores/workspace'
import type { Countdown } from '@/lib/types'

const store = useWorkspaceStore()
const editingId = ref('')
const showForm = ref(false)
const localNow = new Date()
const today = `${localNow.getFullYear()}-${String(localNow.getMonth()+1).padStart(2,'0')}-${String(localNow.getDate()).padStart(2,'0')}`
const form = reactive({ title: '', description: '', target_date: today, repeat_type: 'once' as 'once' | 'yearly' })
const datePickerOpen=ref(false)
const selectedDate=computed(() => form.target_date ? parseDate(form.target_date) : undefined)
function selectDate(value:DateValue | undefined){if(!value)return;form.target_date=value.toString();datePickerOpen.value=false}
const entries = computed(() => [...store.countdowns].sort((a,b) => countdownStatus(a).days - countdownStatus(b).days))
const focusedId=ref('')
watch(()=>store.searchTarget,async target=>{if(target?.kind==='countdown'){focusedId.value=target.id;await nextTick();document.getElementById(`countdown-${target.id}`)?.scrollIntoView({behavior:'smooth',block:'center'});store.searchTarget=null}},{immediate:true})

function resetForm() { datePickerOpen.value=false; editingId.value=''; form.title=''; form.description=''; form.target_date=today; form.repeat_type='once'; showForm.value=false }
function edit(item:Countdown) { editingId.value=item.id; form.title=item.title; form.description=item.description ?? ''; form.target_date=item.target_date; form.repeat_type=item.repeat_type; showForm.value=true }
async function save() {
  if (!form.title.trim() || !form.target_date) return
  try {
    if (editingId.value) {
      const item=store.countdowns.find(x=>x.id===editingId.value)
      if(item) await api.updateCountdown({...item,title:form.title.trim(),description:form.description.trim()||null,target_date:form.target_date,repeat_type:form.repeat_type})
    } else await api.createCountdown({...form,title:form.title.trim(),description:form.description.trim()||null})
    await store.reload(); store.notify('倒数日已保存'); resetForm()
  } catch(error) { store.notify(`保存失败：${String(error)}`) }
}
async function remove(item:Countdown) { try{await api.deleteCountdown(item.id);await store.reload();store.notify('已移入回收站')}catch(e){store.notify(String(e))} }
</script>

<template>
  <div class="flex flex-col gap-8 pb-10">
    <div class="flex items-center justify-end gap-2" aria-label="页面操作"><Button @click="showForm=!showForm"><Plus data-icon="inline-start" />添加日期</Button></div>
    <Card v-if="showForm" class="p-6">
      <div class="mb-5 flex items-center justify-between"><h2 class="font-semibold">{{ editingId ? '编辑日期' : '添加日期' }}</h2><Button variant="ghost" size="sm" @click="resetForm">取消</Button></div>
      <FieldGroup>
          <Field><FieldLabel for="countdown-title">名称</FieldLabel><Input id="countdown-title" v-model="form.title" placeholder="例如：生日、考试、纪念日" /></Field>
          <Field><FieldLabel for="countdown-date">日期</FieldLabel><Popover v-model:open="datePickerOpen"><PopoverTrigger as-child><Button id="countdown-date" variant="outline" class="w-full justify-start"><CalendarDays data-icon="inline-start" />{{ form.target_date ? form.target_date.split('-').join('/') : '选择日期' }}</Button></PopoverTrigger><PopoverContent class="w-auto p-0" align="start"><Calendar :model-value="selectedDate" :default-placeholder="selectedDate" locale="zh-CN" layout="month-and-year" initial-focus @update:model-value="selectDate" /></PopoverContent></Popover></Field>
          <FieldSet><FieldLegend variant="label">重复方式</FieldLegend><RadioGroup v-model="form.repeat_type" aria-label="重复方式"><Field orientation="horizontal"><RadioGroupItem id="countdown-once" value="once" /><FieldLabel for="countdown-once">一次性日期</FieldLabel></Field><Field orientation="horizontal"><RadioGroupItem id="countdown-yearly" value="yearly" /><FieldLabel for="countdown-yearly">每年重复</FieldLabel></Field></RadioGroup></FieldSet>
          <Field><FieldLabel for="countdown-description">说明（可选）</FieldLabel><Textarea id="countdown-description" v-model="form.description" class="min-h-24 resize-y" :rows="4" placeholder="写一点补充信息" /></Field>
        <div class="flex justify-end"><Button @click="save">保存日期</Button></div>
      </FieldGroup>
    </Card>
    <div v-if="entries.length" class="grid grid-cols-2 gap-5">
      <Card v-for="item in entries" :id="`countdown-${item.id}`" :key="item.id" class="flex min-h-48 flex-col justify-between p-6 transition-shadow" :class="focusedId===item.id?'ring-2 ring-primary':''">
        <div class="flex items-start justify-between gap-4">
          <div class="min-w-0"><div class="mb-2 flex items-center gap-2"><CalendarDays class="text-primary" /><Badge>{{ item.repeat_type==='yearly'?'每年重复':'一次性' }}</Badge></div><h2 class="truncate text-lg font-semibold">{{ item.title }}</h2><p class="mt-1 text-sm text-muted-foreground">{{ item.target_date }}<span v-if="item.description"> · {{ item.description }}</span></p></div>
          <div class="flex shrink-0 gap-1"><Button variant="ghost" size="icon" aria-label="编辑" @click="edit(item)"><Pencil /></Button><ConfirmDialog title="删除倒数日？" :description="`确认删除“${item.title}”？内容将移入回收站，可在设置中恢复。`" confirm-text="删除倒数日" destructive @confirm="remove(item)"><template #trigger><Button variant="ghost" size="icon" :aria-label="`删除 ${item.title}`"><Trash2 /></Button></template></ConfirmDialog></div>
        </div>
        <div class="mt-5 flex items-end justify-between"><span class="text-sm text-muted-foreground">{{ item.repeat_type==='yearly'?'下一次':'目标日状态' }}</span><p class="text-xl font-semibold text-primary">{{ countdownStatus(item).text }}</p></div>
      </Card>
    </div>
    <Card v-else-if="!showForm" class="flex min-h-72 flex-col items-center justify-center p-8 text-center">
      <div class="mb-4 flex size-14 items-center justify-center rounded-2xl bg-primary/10 text-primary"><RotateCcw /></div><h2 class="text-lg font-semibold">还没有倒数日</h2><p class="mt-2 max-w-sm text-sm text-muted-foreground">添加生日、纪念日或考试日期，看看距离重要时刻还有多久。</p><Button class="mt-5" variant="outline" @click="showForm=true"><Plus data-icon="inline-start" />添加第一个日期</Button>
    </Card>
  </div>
</template>
