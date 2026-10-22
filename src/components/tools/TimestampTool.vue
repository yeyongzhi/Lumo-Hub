<script setup lang="ts">
import { computed, onUnmounted, ref, shallowRef } from 'vue'
import { CalendarDays } from '@lucide/vue'
import { DateFormatter, getLocalTimeZone, today, type DateValue } from '@internationalized/date'
import { Calendar } from '@/components/ui/calendar'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
const now=ref(Date.now()),stamp=ref(''),unit=ref('seconds'),time=ref('00:00:00'),message=ref(''),result=ref(''),dateResult=ref('')
const date=shallowRef<DateValue>(),datePickerOpen=ref(false)
const defaultDate=today(getLocalTimeZone())
const dateFormatter=new DateFormatter('zh-CN',{dateStyle:'long'})
const dateLabel=computed(()=>date.value?dateFormatter.format(date.value.toDate(getLocalTimeZone())):'选择日期')
function selectDate(value:DateValue|undefined){date.value=value;if(value)datePickerOpen.value=false;dateResult.value=''}
const timer=setInterval(()=>now.value=Date.now(),1000);onUnmounted(()=>clearInterval(timer))
const current=computed(()=>Math.floor(now.value/1000).toString())
function toDate(){try{if(!/^-?\d+$/.test(stamp.value.trim()))throw new Error('请输入整数时间戳');const value=Number(stamp.value)*(unit.value==='seconds'?1000:1);const d=new Date(value);if(!Number.isFinite(value)||isNaN(d.getTime()))throw new Error('时间超出范围');result.value=`本地：${d.toLocaleString()}\nUTC：${d.toISOString()}`;message.value=''}catch(e){result.value='';message.value=String(e)}}
function toStamp(){const d=date.value?new Date(`${date.value.toString()}T${time.value}`):new Date(NaN);if(!date.value||!time.value||isNaN(d.getTime())){dateResult.value='';message.value='请选择有效的本地日期时间';return}dateResult.value=`秒：${Math.floor(d.getTime()/1000)}\n毫秒：${d.getTime()}`;message.value=''}
async function copy(value:string){try{await invoke('clipboard_copy',{text:value});message.value='已复制'}catch(e){message.value=String(e)}}
</script>
<template><div class="flex flex-col gap-6"><section class="flex items-center gap-3 rounded-xl border bg-card p-4"><div class="flex-1"><p class="text-xs text-muted-foreground">当前 Unix 时间戳（秒）</p><p class="mt-2 font-mono text-2xl">{{current}}</p></div><Button variant="outline" @click="copy(current)">复制</Button></section><FieldGroup><Field><FieldLabel for="timestamp">时间戳 → 日期</FieldLabel><Tabs v-model="unit"><TabsList><TabsTrigger value="seconds">秒</TabsTrigger><TabsTrigger value="milliseconds">毫秒</TabsTrigger></TabsList></Tabs><Input id="timestamp" v-model="stamp" placeholder="输入时间戳" @keydown.enter="toDate"/><div class="flex gap-2"><Button @click="toDate">转换日期</Button><Button variant="outline" @click="stamp=String(unit==='seconds'?Math.floor(now/1000):now);toDate()">填入当前时间</Button></div><pre v-if="result" class="whitespace-pre-wrap rounded-lg bg-muted p-3 text-sm">{{result}}</pre><Button v-if="result" variant="outline" class="self-start" @click="copy(result)">复制结果</Button></Field><Field><FieldLabel for="date">本地日期 → 时间戳</FieldLabel><div class="grid gap-3 sm:grid-cols-2"><Popover v-model:open="datePickerOpen"><PopoverTrigger as-child><Button id="date" variant="outline" class="w-full justify-start" aria-label="选择日期"><CalendarDays data-icon="inline-start"/>{{dateLabel}}</Button></PopoverTrigger><PopoverContent class="w-auto p-0" align="start"><Calendar :model-value="date" :default-placeholder="date??defaultDate" locale="zh-CN" layout="month-and-year" initial-focus @update:model-value="selectDate"/></PopoverContent></Popover><Input id="time" v-model="time" type="time" step="1" aria-label="本地时间（时分秒）"/></div><Button class="self-start" @click="toStamp">转换时间戳</Button><pre v-if="dateResult" class="whitespace-pre-wrap rounded-lg bg-muted p-3 text-sm">{{dateResult}}</pre><Button v-if="dateResult" variant="outline" class="self-start" @click="copy(dateResult)">复制结果</Button></Field></FieldGroup><p v-if="message" role="status" class="text-sm text-muted-foreground">{{message}}</p></div></template>
