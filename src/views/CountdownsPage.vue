<script setup lang="ts">
import { writeLocal } from '@/lib/local-files'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { FieldSet, FieldLegend } from '@/components/ui/field'

import { computed, onMounted, onUnmounted, nextTick, reactive, ref, watch } from 'vue'
import { CalendarDays, Pencil, Plus, RotateCcw, Trash2, LayoutGrid, List, Ellipsis } from '@lucide/vue'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Field } from '@/components/ui/field'
import { FieldGroup } from '@/components/ui/field'
import { FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Checkbox } from '@/components/ui/checkbox'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem } from '@/components/ui/dropdown-menu'
import { Calendar } from '@/components/ui/calendar'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { parseDate, type DateValue } from '@internationalized/date'
import { Textarea } from '@/components/ui/textarea'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { useCurrentDay } from '@/lib/use-current-day'
import { format } from 'date-fns'
import { countdownTarget, countdownStatus } from '@/lib/dates'
import { api } from '@/lib/api'
import { useWorkspaceStore } from '@/stores/workspace'
import type { Countdown } from '@/lib/types'

const store = useWorkspaceStore()
const editingId = ref('')
const showForm = ref(false)
const now = useCurrentDay()
const today = computed(()=>format(now.value,'yyyy-MM-dd'))
const form = reactive({ title: '', description: '', target_date: today.value, repeat_type: 'once' as 'once' | 'yearly' })
const datePickerOpen=ref(false)
const selectedDate=computed(() => form.target_date ? parseDate(form.target_date) : undefined)
function selectDate(value:DateValue | undefined){if(!value)return;form.target_date=value.toString();datePickerOpen.value=false}
const reminderDays=ref<number[]>([])
const dateStatus=ref('all')
watch(dateStatus,(value,previous)=>{if(!value)dateStatus.value=previous||'all'})
const query=ref(''), startDate=ref(''), endDate=ref('')
const batchMode=ref(false), selectedIds=ref<string[]>([]), batchBusy=ref(false), deleteBatch=ref(false)
watch([query,startDate,endDate,dateStatus],()=>{if(!batchBusy.value)selectedIds.value=[]})
watch([showForm,editingId],async ([open])=>{if(open){await nextTick();document.getElementById('countdown-form')?.scrollIntoView({block:'nearest'})}})
const displayMode=ref<'grid'|'list'>(store.settings.client?.['lumo-countdown-display']==='list'?'list':'grid')
function setDisplayMode(value:unknown){if(value==='grid'||value==='list')displayMode.value=value}
watch(displayMode,value=>void writeLocal('lumo-countdown-display',value).catch(e=>store.notify(String(e))))
const invalidRange=computed(()=>!!startDate.value&&!!endDate.value&&startDate.value>endDate.value)
const entries = computed(() => store.countdowns.filter(item=>(dateStatus.value==='all'||(dateStatus.value==='past'?countdownStatus(item,now.value).past:!countdownStatus(item,now.value).past))&&(item.title+' '+(item.description??'')).toLocaleLowerCase().includes(query.value.trim().toLocaleLowerCase())&&(!startDate.value||format(countdownTarget(item,now.value),'yyyy-MM-dd')>=startDate.value)&&(!endDate.value||format(countdownTarget(item,now.value),'yyyy-MM-dd')<=endDate.value)).sort((a,b) => Number(countdownStatus(a,now.value).past)-Number(countdownStatus(b,now.value).past)||countdownStatus(a,now.value).days-countdownStatus(b,now.value).days))
watch(()=>store.searchTarget,async target=>{if(target?.kind==='countdown'){dateStatus.value='all';query.value='';startDate.value='';endDate.value='';await nextTick();document.getElementById(`countdown-${target.id}`)?.scrollIntoView({behavior:'smooth',block:'center'});store.searchTarget=null}},{immediate:true})

function resetForm() { datePickerOpen.value=false; editingId.value=''; reminderDays.value=[]; form.title=''; form.description=''; form.target_date=today.value; form.repeat_type='once'; showForm.value=false }
function edit(item:Countdown) { editingId.value=item.id; reminderDays.value=[...(store.settings.product.reminders[item.id]||[])]; form.title=item.title; form.description=item.description ?? ''; form.target_date=item.target_date; form.repeat_type=item.repeat_type; showForm.value=true }
async function save() {
  if (!form.title.trim() || !form.target_date) return
  try {
    let savedId=editingId.value
    if (editingId.value) {
      const item=store.countdowns.find(x=>x.id===editingId.value)
      if(item) await api.updateCountdown({...item,title:form.title.trim(),description:form.description.trim()||null,target_date:form.target_date,repeat_type:form.repeat_type})
    } else savedId=(await api.createCountdown({...form,title:form.title.trim(),description:form.description.trim()||null})).id
    await store.setSetting('product',{...store.settings.product,reminders:{...store.settings.product.reminders,[savedId]:reminderDays.value}})
    await store.reload(); store.notify('倒数日已保存'); resetForm()
  } catch(error) { store.notify(`保存失败：${String(error)}`) }
}
const allSelected=computed(()=>entries.value.length>0&&entries.value.every(item=>selectedIds.value.includes(item.id)))
function toggleSelection(id:string){selectedIds.value=selectedIds.value.includes(id)?selectedIds.value.filter(value=>value!==id):[...selectedIds.value,id]}
function toggleManagement(){batchMode.value=!batchMode.value;selectedIds.value=[]}
onMounted(()=>window.addEventListener('lumo-new',addDate))
onUnmounted(()=>window.removeEventListener('lumo-new',addDate))
function addDate(){resetForm();showForm.value=true}
async function removeSelected(){
  if(batchBusy.value)return
  batchBusy.value=true
  let removed=0
  try{
    for(const id of [...selectedIds.value]){await api.deleteCountdown(id);selectedIds.value=selectedIds.value.filter(value=>value!==id);removed++}
    store.notify('已将 '+removed+' 个倒数日移入回收站')
  }catch(e){store.notify('已删除 '+removed+' 项，其余未完成：'+String(e))}
  finally{try{await store.reload()}catch(e){store.notify(String(e))}batchBusy.value=false}
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <div class="flex shrink-0 flex-wrap items-center gap-3" aria-label="倒数日筛选与操作">
      <div class="flex min-w-40 flex-1 items-center gap-3"><Input v-model="query" class="min-w-0" placeholder="搜索名称、内容" aria-label="搜索倒数日名称和内容" /></div>
      <div class="flex shrink-0 items-center gap-2" role="group" aria-label="日期范围筛选"><Input v-model="startDate" type="date" class="h-8 w-36" aria-label="起始日期" :max="endDate||undefined" /><span class="text-xs text-muted-foreground">至</span><Input v-model="endDate" type="date" class="h-8 w-36" aria-label="结束日期" :min="startDate||undefined" /><Button v-if="startDate||endDate" variant="ghost" size="icon-sm" aria-label="清除日期筛选" @click="startDate='';endDate=''"><RotateCcw /></Button></div>
      <Select v-model="dateStatus"><SelectTrigger class="w-28" aria-label="日期状态"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem value="all">全部</SelectItem><SelectItem value="active">未结束</SelectItem><SelectItem value="past">已结束</SelectItem></SelectGroup></SelectContent></Select>
      <TooltipProvider :delay-duration="200"><Tabs :model-value="displayMode" @update:model-value="setDisplayMode"><TabsList aria-label="显示方式"><Tooltip><TooltipTrigger as-child><TabsTrigger value="grid" aria-label="网格展示"><LayoutGrid /></TabsTrigger></TooltipTrigger><TooltipContent>网格展示</TooltipContent></Tooltip><Tooltip><TooltipTrigger as-child><TabsTrigger value="list" aria-label="列表展示"><List /></TabsTrigger></TooltipTrigger><TooltipContent>列表展示</TooltipContent></Tooltip></TabsList></Tabs></TooltipProvider>
      <div class="flex shrink-0 items-center gap-px"><Button size="sm" class="rounded-r-none" @click="addDate"><Plus data-icon="inline-start" />添加倒数日</Button><DropdownMenu><DropdownMenuTrigger as-child><Button size="icon-sm" class="rounded-l-none" aria-label="更多倒数日操作"><Ellipsis /></Button></DropdownMenuTrigger><DropdownMenuContent align="end"><DropdownMenuGroup><DropdownMenuItem :disabled="batchBusy" @select="toggleManagement"><Pencil />{{ batchMode?'完成管理':'管理倒数日' }}</DropdownMenuItem></DropdownMenuGroup></DropdownMenuContent></DropdownMenu></div>
    </div>
    <p v-if="invalidRange" class="shrink-0 text-sm text-destructive">开始日期不能晚于结束日期。</p>
    <div v-if="batchMode&&!showForm" class="flex shrink-0 items-center gap-3 rounded-xl bg-muted p-3"><Checkbox :model-value="allSelected" :disabled="batchBusy||!entries.length" aria-label="选择当前筛选结果" @update:model-value="value=>selectedIds=value===true?entries.map(item=>item.id):[]" /><span class="flex-1 text-sm">已选择 {{ selectedIds.length }} 项</span><Button variant="destructive" size="sm" :disabled="!selectedIds.length||batchBusy" @click="deleteBatch=true"><Trash2 data-icon="inline-start" />{{ batchBusy?'正在删除…':'删除所选' }}</Button><Button variant="outline" size="sm" :disabled="batchBusy" @click="toggleManagement">完成管理</Button></div>
    <div class="relative min-h-0 flex-1 overflow-hidden"><ScrollArea v-if="entries.length||showForm" class="absolute inset-0 h-full w-full" aria-label="倒数日内容"><div class="flex min-w-0 flex-col gap-5 pr-3 pb-3">
    <Card v-if="showForm" id="countdown-form" class="p-6">
      <div class="mb-5 flex items-center justify-between"><h2 class="text-2xl font-semibold">{{ editingId ? '编辑倒数日' : '添加倒数日' }}</h2><Button variant="ghost" size="sm" @click="resetForm">取消</Button></div>
      <FieldGroup>
          <Field><FieldLabel for="countdown-title">名称</FieldLabel><Input id="countdown-title" v-model="form.title" placeholder="例如：生日、考试、纪念日" /></Field>
          <Field><FieldLabel for="countdown-date">日期</FieldLabel><Popover v-model:open="datePickerOpen"><PopoverTrigger as-child><Button id="countdown-date" variant="outline" class="w-full justify-start"><CalendarDays data-icon="inline-start" />{{ form.target_date ? form.target_date.split('-').join('/') : '选择日期' }}</Button></PopoverTrigger><PopoverContent class="w-auto p-0" align="start"><Calendar :model-value="selectedDate" :default-placeholder="selectedDate" locale="zh-CN" layout="month-and-year" initial-focus @update:model-value="selectDate" /></PopoverContent></Popover></Field>
          <FieldSet><FieldLegend variant="label">重复方式</FieldLegend><RadioGroup v-model="form.repeat_type" aria-label="重复方式"><Field orientation="horizontal"><RadioGroupItem id="countdown-once" value="once" /><FieldLabel for="countdown-once">一次性日期</FieldLabel></Field><Field orientation="horizontal"><RadioGroupItem id="countdown-yearly" value="yearly" /><FieldLabel for="countdown-yearly">每年重复</FieldLabel></Field></RadioGroup></FieldSet>
          <FieldSet><FieldLegend variant="label">提醒（可多选）</FieldLegend><div class="flex flex-wrap gap-4"><Field v-for="option in [{days:0,label:'当天'},{days:1,label:'提前一天'},{days:7,label:'提前一周'}]" :key="option.days" orientation="horizontal" class="w-auto"><Checkbox :id="`reminder-${option.days}`" :model-value="reminderDays.includes(option.days)" @update:model-value="value=>reminderDays=value===true?[...new Set([...reminderDays,option.days])]:reminderDays.filter(day=>day!==option.days)" /><FieldLabel :for="`reminder-${option.days}`">{{ option.label }}</FieldLabel></Field></div><p class="text-xs text-muted-foreground">每天本地时间 {{ String(store.settings.product.reminder_hour).padStart(2,'0') }}:00 检查。应用关闭或电脑关机时不会发送；应用再次运行后会补发最近错过的一天。可在设置中发送测试通知。</p></FieldSet>
          <Field><FieldLabel for="countdown-description">说明（可选）</FieldLabel><Textarea id="countdown-description" v-model="form.description" class="min-h-24 resize-y" :rows="4" placeholder="写一点补充信息" /></Field>
        <div class="flex justify-end"><Button @click="save">保存倒数日</Button></div>
      </FieldGroup>
    </Card>
      <div v-else-if="entries.length" :class="displayMode==='grid'?'grid grid-cols-[repeat(auto-fill,minmax(min(100%,22rem),1fr))] gap-5':'flex flex-col gap-3'">
        <Card v-for="item in entries" :id="'countdown-'+item.id" :key="item.id" class="flex min-w-0 p-5" :class="displayMode==='grid'?'min-h-48 flex-col justify-between':'flex-row items-center gap-5'">
          <div class="flex min-w-0 flex-1 items-start gap-3"><Checkbox v-if="batchMode" class="mt-1" :model-value="selectedIds.includes(item.id)" :disabled="batchBusy" :aria-label="'选择 '+item.title" @update:model-value="toggleSelection(item.id)" /><div class="min-w-0 flex-1"><div class="flex items-start justify-between gap-3"><div class="flex min-w-0 items-center gap-2"><CalendarDays class="size-5 shrink-0 text-primary" /><h2 class="truncate text-lg font-semibold">{{ item.title }}</h2></div><Badge class="shrink-0">{{ item.repeat_type==='yearly'?'每年重复':'一次性' }}</Badge></div><p class="mt-3 text-xl font-medium tabular-nums">{{ format(countdownTarget(item,now),'yyyy-MM-dd') }}{{ item.repeat_type==='yearly'?' · 下一次':'' }}</p><p v-if="item.description" class="mt-1 line-clamp-2 break-words text-sm text-muted-foreground">{{ item.description }}</p></div><Button v-if="batchMode" variant="ghost" size="icon" :disabled="batchBusy" :aria-label="'编辑 '+item.title" @click="edit(item)"><Pencil /></Button></div>
          <div class="flex shrink-0 items-end justify-between gap-4" :class="displayMode==='grid'?'mt-5':'flex-col'"><div class="flex flex-col gap-1"><span class="text-sm text-muted-foreground">距离目标日</span><span class="text-xs text-muted-foreground">提醒：{{ (store.settings.product.reminders[item.id]||[]).map(day=>day===0?'当天':day===1?'提前一天':'提前一周').join('、')||'未设置' }}</span></div><p class="whitespace-nowrap text-xl font-semibold text-primary">{{ countdownStatus(item,now).text }}</p></div>
        </Card>
      </div>

    </div></ScrollArea>
      <Card v-else class="flex h-full min-h-0 flex-col items-center justify-center p-8 text-center"><RotateCcw class="mb-4 size-8 text-primary" /><h2 class="text-lg font-semibold">{{ store.countdowns.length?'没有匹配的倒数日':'还没有倒数日' }}</h2><p class="mt-2 text-sm text-muted-foreground">{{ store.countdowns.length?'试试调整搜索关键词或日期范围。':'添加生日、纪念日或考试日期。' }}</p></Card>
    </div>
    <ConfirmDialog v-model:open="deleteBatch" title="删除所选倒数日？" :description="'将删除 '+selectedIds.length+' 个倒数日。内容移入回收站，可在设置中恢复。'" confirm-text="删除所选" destructive @confirm="removeSelected" />
  </div>
</template>
