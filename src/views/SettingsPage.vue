<script setup lang="ts">
import { askConfirmation } from '@/lib/confirm'
import AboutPage from '@/views/AboutPage.vue'
import EnhancementSettings from '@/components/EnhancementSettings.vue'
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Field, FieldGroup, FieldLabel, FieldDescription } from '@/components/ui/field'
import { computed, nextTick, ref, watch } from 'vue'
import { CircleHelp, Check, Database, Download, FolderCog, Settings, Sun, Moon, Monitor, Upload, Keyboard, ShieldCheck, Trash2, AppWindow, LayoutGrid, List } from '@lucide/vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Card, CardContent } from '@/components/ui/card'
import { useWorkspaceStore } from '@/stores/workspace'
import type { AppSettings } from '@/lib/types'
import { api } from '@/lib/api'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'

const store=useWorkspaceStore()
const busy=ref(false)
const section=ref('appearance')
const props=defineProps<{request?:{checkUpdate:boolean;id:number}|null}>()
const aboutPage=ref<InstanceType<typeof AboutPage>|null>(null)
watch(()=>props.request,async request=>{
  if(!request)return
  section.value='about'
  await nextTick()
  if(request.checkUpdate)void aboutPage.value?.checkUpdate()
},{immediate:true})
const settingsMenu=[{id:'appearance',label:'外观',icon:Settings},{id:'library',label:'应用库',icon:AppWindow},{id:'data',label:'数据与备份',icon:Database},{id:'launcher',label:'快捷启动',icon:Keyboard},{id:'backup',label:'自动备份',icon:ShieldCheck},{id:'trash',label:'回收站',icon:Trash2},{id:'about',label:'关于',icon:CircleHelp}]
const fontSizes=computed(()=>[...new Set([12,14,16,18,20,24,store.settings.font_size])].sort((a,b)=>a-b))
const radii=[{value:0,label:'直角 · 0 px'},{value:4,label:'微圆角 · 4 px'},{value:8,label:'小圆角 · 8 px'},{value:12,label:'中圆角 · 12 px'},{value:16,label:'大圆角 · 16 px（默认）'},{value:24,label:'超大圆角 · 24 px'}]
const modes=[{id:'light',label:'浅色',icon:Sun},{id:'dark',label:'深色',icon:Moon},{id:'system',label:'跟随系统',icon:Monitor}] as const
const accents=[{id:'red',label:'红',className:'bg-red-500'},{id:'orange',label:'橙',className:'bg-orange-500'},{id:'yellow',label:'黄',className:'bg-yellow-400'},{id:'green',label:'绿',className:'bg-green-500'},{id:'blue',label:'蓝',className:'bg-blue-500'},{id:'indigo',label:'靛',className:'bg-indigo-500'},{id:'violet',label:'紫',className:'bg-violet-500'}] as const
async function setMode(mode:AppSettings['theme_mode']){try{await store.setSetting('theme_mode',mode)}catch{}}
async function setAccent(accent:AppSettings['accent']){try{await store.setSetting('accent',accent)}catch{}}
async function setFont(value:unknown){try{await store.setSetting('font_size',Number(value))}catch{}}
async function setRadius(value:unknown){try{await store.setSetting('border_radius',Number(value))}catch{}}
async function setLibraryDisplay(value:unknown){if(value!=='grid'&&value!=='list')return;try{await store.saveClient('lumo-library-display',value)}catch(e){store.notify(`保存应用库设置失败：${String(e)}`)}}
async function setLibraryDensity(value:unknown){if(value!=='compact'&&value!=='standard'&&value!=='comfortable')return;try{await store.saveClient('lumo-library-density',value)}catch(e){store.notify(`保存应用库设置失败：${String(e)}`)}}
async function chooseDataPath(){const result=await open({directory:true,multiple:false});if(typeof result!=='string')return;if(!await askConfirmation(`将全部设置、笔记、便签和其他数据迁移到：\n${result}\n\n目标目录需要为空；迁移前自动备份，原数据保留。继续吗？`))return;busy.value=true;try{await store.flushPendingSaves();store.bootstrap=await api.moveDataDirectory(result);await store.reload();store.notify('数据迁移完成，后续使用新目录')}catch(e){store.notify(`迁移失败：${String(e)}`)}finally{busy.value=false}}
async function createBackup(){const path=await save({defaultPath:`Lumo-Hub-备份-${new Date().toISOString().slice(0,10)}.lumo`,filters:[{name:'Lumo-Hub 备份',extensions:['lumo']}]});if(!path)return;busy.value=true;try{await store.flushPendingSaves();await api.exportBackup(path);store.notify('备份已导出')}catch(e){store.notify(`备份失败：${String(e)}`)}finally{busy.value=false}}
async function restoreBackup(){const path=await open({defaultPath:`${store.bootstrap.data_dir}/backups`,multiple:false,filters:[{name:'Lumo-Hub 备份',extensions:['lumo']}]});if(typeof path!=='string')return;if(!await askConfirmation('恢复备份会替换当前笔记、便签、倒数日、应用库数据和外观设置。恢复前会自动创建当前数据备份。确认继续？'))return;busy.value=true;try{await store.flushPendingSaves();await api.restoreBackup(path);const current=await api.bootstrap();store.bootstrap=current.bootstrap;store.settings=current.settings;await store.reload();store.notify('备份已恢复')}catch(e){store.notify(`恢复失败：${String(e)}`)}finally{busy.value=false}}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col items-stretch gap-4 overflow-hidden">
    <nav class="flex shrink-0 items-center gap-1 overflow-x-auto rounded-xl border border-border bg-card p-2" aria-label="设置分类">
      <Button v-for="item in settingsMenu" :key="item.id" variant="ghost" class="h-10 shrink-0 gap-2 rounded-lg px-3 text-sm" :class="section===item.id?'bg-primary/10 font-semibold text-primary':'text-muted-foreground'" :aria-current="section===item.id?'page':undefined" @click="section=item.id"><component :is="item.icon" />{{ item.label }}</Button>
    </nav>
    <Card class="min-h-0 min-w-0 flex-1 gap-0 overflow-hidden py-0" data-settings-panel>
      <CardContent v-if="section==='trash'" class="flex min-h-0 flex-1 flex-col overflow-y-auto py-6"><EnhancementSettings section="trash" @restore-backup="restoreBackup" /></CardContent>
      <ScrollArea v-else :key="section" class="min-h-0 flex-1" data-settings-scroll>
      <CardContent class="py-6">
    <div v-if="section=== 'appearance'" class="flex min-w-0 flex-col gap-6">
    
      <div class="flex flex-col gap-7">
        <section><h3 class="mb-3 text-base font-semibold">外观模式</h3><div class="grid max-w-2xl grid-cols-3 gap-3"><Button v-for="mode in modes" :key="mode.id" variant="outline" class="h-auto justify-start rounded-xl py-3" :class="store.settings.theme_mode===mode.id?'border-primary bg-primary/5 text-primary':''" @click="setMode(mode.id)"><component :is="mode.icon" />{{ mode.label }}<Check v-if="store.settings.theme_mode===mode.id" class="ml-auto" /></Button></div></section>
        <section><h3 class="mb-3 text-base font-semibold">强调色</h3><div class="flex flex-wrap gap-3"><Button v-for="accent in accents" :key="accent.id" variant="outline" size="sm" class="rounded-full" :class="store.settings.accent===accent.id?'ring-2 ring-primary ring-offset-2 ring-offset-background':''" :aria-label="`${accent.label}色主题`" @click="setAccent(accent.id)"><span class="size-4 rounded-full" :class="accent.className"></span>{{ accent.label }}</Button></div></section>
        <FieldGroup>
          <Field><FieldLabel class="text-base font-semibold" for="global-radius">全局圆角</FieldLabel><Select :model-value="String(store.settings.border_radius ?? 16)" @update:model-value="setRadius"><SelectTrigger id="global-radius" class="w-full max-w-xs"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem v-for="option in radii" :key="option.value" :value="String(option.value)">{{ option.label }}</SelectItem></SelectGroup></SelectContent></Select><FieldDescription>统一调整卡片、按钮、输入框和弹窗的圆角。</FieldDescription></Field>
          <Field><FieldLabel class="text-base font-semibold" for="global-font-size">全局字体大小</FieldLabel><Select :model-value="String(store.settings.font_size)" @update:model-value="setFont"><SelectTrigger id="global-font-size" class="w-full max-w-xs"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem v-for="size in fontSizes" :key="size" :value="String(size)">{{ size }} px{{ size===14?'（默认）':'' }}</SelectItem></SelectGroup></SelectContent></Select><FieldDescription>选择适合阅读的字号，整个应用立即更新。</FieldDescription></Field>
        </FieldGroup>
      </div>
    </div>
    <div v-else-if="section==='library'" class="flex min-w-0 flex-col gap-6">
      <FieldGroup>
        <Field><div class="min-w-0 flex-1"><FieldLabel class="text-base font-semibold">显示方式</FieldLabel><FieldDescription>选择网格或列表布局。</FieldDescription></div><Tabs class="max-w-fit" :model-value="String(store.settings.client?.['lumo-library-display']??'grid')" @update:model-value="setLibraryDisplay"><TabsList aria-label="应用库显示方式"><TabsTrigger value="grid"><LayoutGrid />网格</TabsTrigger><TabsTrigger value="list"><List />列表</TabsTrigger></TabsList></Tabs></Field>
        <Field><div class="min-w-0 flex-1"><FieldLabel class="text-base font-semibold" for="library-density">网格密度</FieldLabel><FieldDescription>调整网格布局中每个启动项的占用空间。</FieldDescription></div><Select :model-value="String(store.settings.client?.['lumo-library-density']??'standard')" @update:model-value="setLibraryDensity"><SelectTrigger id="library-density" class="w-full max-w-xs"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem value="compact">紧凑</SelectItem><SelectItem value="standard">标准</SelectItem><SelectItem value="comfortable">宽松</SelectItem></SelectGroup></SelectContent></Select></Field>
      </FieldGroup>
    </div>
    <div v-else-if="section=== 'data'" class="flex min-w-0 flex-col gap-6">
      <div class="flex flex-col gap-5">
        <div class="rounded-2xl bg-muted/60 p-4"><p class="text-xs text-muted-foreground">当前数据目录</p><p class="mt-1 break-all font-mono text-sm">{{ store.bootstrap.data_dir||'读取中…' }}</p><p class="mt-2 text-xs text-muted-foreground">{{ store.bootstrap.data_follows_install?'默认保存在程序安装目录，可迁移到自定义目录。':'当前使用独立数据目录；可迁移到其他位置，后续启动会使用所选目录。' }}</p></div>
        <div><Button variant="outline" :disabled="busy" @click="chooseDataPath"><FolderCog data-icon="inline-start" />迁移数据目录</Button><p class="mt-2 break-all text-xs text-muted-foreground">默认目录：{{ store.bootstrap.default_data_dir }}</p></div><div class="border-t border-border"></div>
        <div><h3 class="mb-1 text-base font-semibold">备份与恢复</h3><p class="mb-3 text-sm text-muted-foreground">备份会包含数据库和用户资源。恢复前会自动备份当前数据。</p><div class="flex gap-2"><Button variant="outline" :disabled="busy" @click="createBackup"><Download data-icon="inline-start" />导出备份</Button><Button variant="outline" :disabled="busy" @click="restoreBackup"><Upload data-icon="inline-start" />恢复备份</Button></div></div>
      </div>
    </div>
    <div v-else-if="['launcher','backup'].includes(section)" :key="section" class="flex min-w-0 flex-col gap-6"><EnhancementSettings :section="section" @restore-backup="restoreBackup" /></div>
    <div v-else-if="section=== 'about'" class="flex min-w-0 flex-col gap-6"><AboutPage ref="aboutPage" /></div>
    </CardContent>
    </ScrollArea>
    </Card>
  </div>
</template>
