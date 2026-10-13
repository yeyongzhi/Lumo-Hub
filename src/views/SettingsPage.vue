<script setup lang="ts">
import { askConfirmation } from '@/lib/confirm'
import AboutPage from '@/views/AboutPage.vue'
import EnhancementSettings from '@/components/EnhancementSettings.vue'
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Field, FieldGroup, FieldLabel, FieldDescription } from '@/components/ui/field'
import { computed, ref } from 'vue'
import { CircleHelp, Check, Database, Download, FolderCog, Settings, Sun, Moon, Monitor, Upload, Keyboard, ShieldCheck, Trash2 } from '@lucide/vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { useWorkspaceStore } from '@/stores/workspace'
import type { AppSettings } from '@/lib/types'
import { api } from '@/lib/api'

const store=useWorkspaceStore()
const busy=ref(false)
const section=ref('appearance')
const settingsMenu=[{id:'appearance',label:'外观',icon:Settings},{id:'data',label:'数据与备份',icon:Database},{id:'launcher',label:'快捷启动',icon:Keyboard},{id:'backup',label:'自动备份',icon:ShieldCheck},{id:'trash',label:'回收站',icon:Trash2},{id:'about',label:'关于',icon:CircleHelp}]
const fontSizes=computed(()=>[...new Set([12,14,16,18,20,24,store.settings.font_size])].sort((a,b)=>a-b))
const sidebarWidths=[100,140,180,200,220,240,260,280]
const radii=[{value:0,label:'直角 · 0 px'},{value:4,label:'微圆角 · 4 px'},{value:8,label:'小圆角 · 8 px'},{value:12,label:'中圆角 · 12 px'},{value:16,label:'大圆角 · 16 px（默认）'},{value:24,label:'超大圆角 · 24 px'}]
const modes=[{id:'light',label:'浅色',icon:Sun},{id:'dark',label:'深色',icon:Moon},{id:'system',label:'跟随系统',icon:Monitor}] as const
const accents=[{id:'red',label:'红',className:'bg-red-500'},{id:'orange',label:'橙',className:'bg-orange-500'},{id:'yellow',label:'黄',className:'bg-yellow-400'},{id:'green',label:'绿',className:'bg-green-500'},{id:'blue',label:'蓝',className:'bg-blue-500'},{id:'indigo',label:'靛',className:'bg-indigo-500'},{id:'violet',label:'紫',className:'bg-violet-500'}] as const
async function setMode(mode:AppSettings['theme_mode']){try{await store.setSetting('theme_mode',mode)}catch{}}
async function setAccent(accent:AppSettings['accent']){try{await store.setSetting('accent',accent)}catch{}}
async function setFont(value:unknown){try{await store.setSetting('font_size',Number(value))}catch{}}
async function setSidebarWidth(value:unknown){try{await store.setSetting('sidebar_width',Number(value))}catch{}}
async function setRadius(value:unknown){try{await store.setSetting('border_radius',Number(value))}catch{}}
async function chooseDataPath(){const result=await open({directory:true,multiple:false});if(typeof result!=='string')return;if(!await askConfirmation(`将全部设置、笔记、便签和其他数据迁移到：\n${result}\n\n目标目录需要为空；迁移前自动备份，原数据保留。继续吗？`))return;busy.value=true;try{await store.flushPendingSaves();store.bootstrap=await api.moveDataDirectory(result);await store.reload();store.notify('数据迁移完成，后续使用新目录')}catch(e){store.notify(`迁移失败：${String(e)}`)}finally{busy.value=false}}
async function createBackup(){const path=await save({defaultPath:`Lumo-Hub-备份-${new Date().toISOString().slice(0,10)}.lumo`,filters:[{name:'Lumo-Hub 备份',extensions:['lumo']}]});if(!path)return;busy.value=true;try{await store.flushPendingSaves();await api.exportBackup(path);store.notify('备份已导出')}catch(e){store.notify(`备份失败：${String(e)}`)}finally{busy.value=false}}
async function restoreBackup(){const path=await open({defaultPath:`${store.bootstrap.data_dir}/backups`,multiple:false,filters:[{name:'Lumo-Hub 备份',extensions:['lumo']}]});if(typeof path!=='string')return;if(!await askConfirmation('恢复备份会替换当前笔记、便签、倒数日、应用库数据和外观设置。恢复前会自动创建当前数据备份。确认继续？'))return;busy.value=true;try{await store.flushPendingSaves();await api.restoreBackup(path);const current=await api.bootstrap();store.bootstrap=current.bootstrap;store.settings=current.settings;await store.reload();store.notify('备份已恢复')}catch(e){store.notify(`恢复失败：${String(e)}`)}finally{busy.value=false}}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-row items-stretch gap-6 overflow-hidden">
    <Card class="min-h-0 w-44 shrink-0 gap-3 overflow-y-auto overscroll-contain p-3"><h2 class="shrink-0 px-2 text-sm font-semibold">设置</h2><nav class="flex w-full shrink-0 flex-col gap-1" aria-label="设置分类"><Button v-for="item in settingsMenu" :key="item.id" variant="ghost" class="h-11 w-full justify-start gap-3 rounded-xl px-3 text-sm" :class="section===item.id?'bg-primary/10 font-semibold text-primary':'text-muted-foreground'" :aria-current="section===item.id?'page':undefined" @click="section=item.id"><component :is="item.icon" />{{ item.label }}</Button></nav></Card>
    <div class="settings-panels min-h-0 min-w-0 flex-1 overflow-hidden">
    <div v-if="section=== 'appearance'" class="settings-panel">
    
    <Card class="p-6">
      <div class="mb-6 flex items-center gap-3"><div class="flex size-11 items-center justify-center rounded-2xl bg-primary/10 text-primary"><Settings /></div><div><h2 class="font-semibold">外观</h2><p class="text-sm text-muted-foreground">更改后立即预览，并在下次启动时保留。</p></div></div>
      <div class="flex flex-col gap-7">
        <section><h3 class="mb-3 text-sm font-medium">外观模式</h3><div class="grid max-w-2xl grid-cols-3 gap-3"><Button v-for="mode in modes" :key="mode.id" variant="outline" class="h-auto justify-start rounded-xl py-3" :class="store.settings.theme_mode===mode.id?'border-primary bg-primary/5 text-primary':''" @click="setMode(mode.id)"><component :is="mode.icon" />{{ mode.label }}<Check v-if="store.settings.theme_mode===mode.id" class="ml-auto" /></Button></div></section>
        <section><h3 class="mb-3 text-sm font-medium">强调色</h3><div class="flex flex-wrap gap-3"><Button v-for="accent in accents" :key="accent.id" variant="outline" size="sm" class="rounded-full" :class="store.settings.accent===accent.id?'ring-2 ring-primary ring-offset-2 ring-offset-background':''" :aria-label="`${accent.label}色主题`" @click="setAccent(accent.id)"><span class="size-4 rounded-full" :class="accent.className"></span>{{ accent.label }}</Button></div></section>
        <FieldGroup>
          <Field><FieldLabel for="sidebar-width">侧边栏宽度</FieldLabel><Select :model-value="String(store.settings.sidebar_width ?? 240)" @update:model-value="setSidebarWidth"><SelectTrigger id="sidebar-width" class="w-full max-w-xs"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem v-for="width in sidebarWidths" :key="width" :value="String(width)">{{ width }} px{{ width===240?'（默认）':'' }}</SelectItem></SelectGroup></SelectContent></Select><FieldDescription>调整展开后的主侧边栏宽度，收缩后的宽度保持一致。</FieldDescription></Field>
          <Field><FieldLabel for="global-radius">全局圆角</FieldLabel><Select :model-value="String(store.settings.border_radius ?? 16)" @update:model-value="setRadius"><SelectTrigger id="global-radius" class="w-full max-w-xs"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem v-for="option in radii" :key="option.value" :value="String(option.value)">{{ option.label }}</SelectItem></SelectGroup></SelectContent></Select><FieldDescription>统一调整卡片、按钮、输入框和弹窗的圆角。</FieldDescription></Field>
          <Field><FieldLabel for="global-font-size">全局字体大小</FieldLabel><Select :model-value="String(store.settings.font_size)" @update:model-value="setFont"><SelectTrigger id="global-font-size" class="w-full max-w-xs"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem v-for="size in fontSizes" :key="size" :value="String(size)">{{ size }} px{{ size===14?'（默认）':'' }}</SelectItem></SelectGroup></SelectContent></Select><FieldDescription>选择适合阅读的字号，整个应用立即更新。</FieldDescription></Field>
        </FieldGroup>
      </div>
    </Card>
    </div>
    <div v-else-if="section=== 'data'" class="settings-panel">
    <Card class="p-6">
      <div class="mb-6 flex items-center gap-3"><div class="flex size-11 items-center justify-center rounded-2xl bg-primary/10 text-primary"><Database /></div><div><h2 class="font-semibold">数据与备份</h2><p class="text-sm text-muted-foreground">数据只保存在本机，可随时备份或迁移。</p></div></div>
      <div class="flex flex-col gap-5">
        <div class="rounded-2xl bg-muted/60 p-4"><p class="text-xs text-muted-foreground">当前数据目录</p><p class="mt-1 break-all font-mono text-sm">{{ store.bootstrap.data_dir||'读取中…' }}</p><p class="mt-2 text-xs text-muted-foreground">{{ store.bootstrap.data_follows_install?'默认保存在程序安装目录，可迁移到自定义目录。':'当前使用独立数据目录；可迁移到其他位置，后续启动会使用所选目录。' }}</p></div>
        <div><Button variant="outline" :disabled="busy" @click="chooseDataPath"><FolderCog data-icon="inline-start" />迁移数据目录</Button><p class="mt-2 break-all text-xs text-muted-foreground">默认目录：{{ store.bootstrap.default_data_dir }}</p></div><div class="border-t border-border"></div>
        <div><h3 class="mb-1 font-medium">备份与恢复</h3><p class="mb-3 text-sm text-muted-foreground">备份会包含数据库和用户资源。恢复前会自动备份当前数据。</p><div class="flex gap-2"><Button variant="outline" :disabled="busy" @click="createBackup"><Download data-icon="inline-start" />导出备份</Button><Button variant="outline" :disabled="busy" @click="restoreBackup"><Upload data-icon="inline-start" />恢复备份</Button></div></div>
      </div>
    </Card>
    </div>
    <div v-else-if="['launcher','backup','trash'].includes(section)" :key="section" class="settings-panel"><EnhancementSettings :section="section" @restore-backup="restoreBackup" /></div>
    <div v-else-if="section=== 'about'" class="settings-panel"><AboutPage /></div>
    </div>
  </div>
</template>

<style scoped>
.settings-panel {
  height: 100%;
  min-height: 0;
}

.settings-panels :deep(.settings-panel > [data-slot="card"]) {
  height: 100%;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
}

.settings-panels :deep(.settings-panel > [data-slot="card"] > *) {
  flex-shrink: 0;
}
</style>
