<script setup lang="ts">
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

import { onMounted, onUnmounted, ref, watch } from 'vue'
import { Keyboard, ShieldCheck, Trash2, RotateCcw } from '@lucide/vue'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import { Field, FieldGroup, FieldLabel, FieldDescription } from '@/components/ui/field'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { api } from '@/lib/api'
import type { BackupStatus, TrashItem } from '@/lib/types'
import { useWorkspaceStore } from '@/stores/workspace'
const props=withDefaults(defineProps<{section?:string}>(),{section:'all'})
const store=useWorkspaceStore()
const emit=defineEmits<{ restoreBackup: [] }>()
const shortcut=ref(store.settings.launcher.shortcut), shortcutError=ref(''), saving=ref(false), backupBusy=ref(false)
const status=ref<BackupStatus>(), trash=ref<TrashItem[]>([]), loading=ref(true), error=ref(''), trashBusy=ref(false)
const purgeOpen=ref(false), purgeId=ref<string|null>(null)
const kindNames={note:'笔记',sticky:'便签',countdown:'倒数日'}
watch(()=>store.settings.launcher.shortcut,value=>shortcut.value=value)
watch(()=>store.settings,()=>void refresh())
let timer:ReturnType<typeof setInterval>
async function refresh(){try{const [s,t,e]=await Promise.all([api.autoBackupStatus(),api.listTrash(),api.launcherStatus()]);status.value=s;trash.value=t;shortcutError.value=e??'';error.value=''}catch(e){error.value=String(e)}finally{loading.value=false}}
onMounted(()=>{void refresh();timer=setInterval(refresh,30000)})
onUnmounted(()=>clearInterval(timer))
async function configure(enabled=store.settings.launcher.enabled,minimalMode=store.settings.launcher.minimal_mode){saving.value=true;try{store.settings=await api.configureLauncher({enabled,shortcut:shortcut.value.trim(),minimal_mode:minimalMode});shortcutError.value='';store.notify('快捷启动设置已保存')}catch(e){shortcutError.value=String(e)}finally{saving.value=false}}
async function backupOptions(value:Partial<typeof store.settings.auto_backup>){try{await store.setSetting('auto_backup',{...store.settings.auto_backup,...value});await refresh()}catch{}}
async function backupNow(){backupBusy.value=true;try{await store.flushPendingSaves();await api.runAutoBackup();await refresh();store.notify('自动备份已创建')}catch(e){store.notify(String(e))}finally{backupBusy.value=false}}
async function restore(item:TrashItem){trashBusy.value=true;try{await api.restoreTrash(item.id);await store.reload();await refresh();store.notify(`${kindNames[item.kind]}已恢复`)}catch(e){store.notify(String(e))}finally{trashBusy.value=false}}
function askPurge(id:string|null){purgeId.value=id;purgeOpen.value=true}
async function purge(){trashBusy.value=true;try{await api.purgeTrash(purgeId.value);await refresh();store.notify('已永久删除')}catch(e){store.notify(String(e))}finally{trashBusy.value=false}}
function date(value:string|null){return value?new Date(value).toLocaleString('zh-CN'):'尚未备份'}
</script>
<template>
  <Card v-if="props.section==='launcher'||props.section==='all'"><CardHeader><CardTitle class="flex items-center gap-2"><Keyboard class="size-5 text-primary"/>快捷启动</CardTitle><CardDescription>程序运行或最小化时，随时唤出简洁面板。关闭程序后快捷键停止生效。</CardDescription></CardHeader><CardContent>
    <FieldGroup><Field orientation="horizontal"><FieldLabel for="launcher-enabled">启用全局快捷键</FieldLabel><Switch id="launcher-enabled" :model-value="store.settings.launcher.enabled" :disabled="saving" @update:model-value="configure"/></Field>
      <Field orientation="horizontal"><div><FieldLabel for="minimal-mode">极简模式</FieldLabel><FieldDescription>启动程序时直接显示搜索、最近内容和应用网格；点击主界面图标进入完整工作台。</FieldDescription></div><Switch id="minimal-mode" :model-value="store.settings.launcher.minimal_mode" :disabled="saving" @update:model-value="value=>configure(store.settings.launcher.enabled,value)"/></Field>
      <Field><FieldLabel for="launcher-shortcut">快捷键</FieldLabel><div class="flex max-w-lg gap-2"><Input id="launcher-shortcut" v-model="shortcut" placeholder="Alt+Space" @keydown.enter="configure()"/><Button :disabled="saving" @click="configure()">保存</Button><Button variant="outline" @click="api.showLauncher().catch(e=>store.notify(String(e)))">预览面板</Button></div><FieldDescription>例如 Alt+Space、Control+Alt+Space。面板支持 ↑ ↓、Enter 和 Esc。</FieldDescription></Field>
      <p v-if="shortcutError" class="text-sm text-destructive" role="alert">{{ shortcutError }}</p>
    </FieldGroup>
  </CardContent></Card>
  <Card v-if="props.section==='backup'||props.section==='all'"><CardHeader><CardTitle class="flex items-center gap-2"><ShieldCheck class="size-5 text-primary"/>自动备份</CardTitle><CardDescription>备份保存在当前数据目录的 backups 文件夹；包含内容、回收站、图标和设置。</CardDescription></CardHeader><CardContent>
    <FieldGroup><Field orientation="horizontal"><FieldLabel for="backup-enabled">启用定期备份</FieldLabel><Switch id="backup-enabled" :model-value="store.settings.auto_backup.enabled" @update:model-value="enabled=>backupOptions({enabled})"/></Field>
    <Field><FieldLabel for="backup-interval">备份周期</FieldLabel><Select :model-value="String(store.settings.auto_backup.interval_hours)" @update:model-value="value=>backupOptions({interval_hours:Number(value)})"><SelectTrigger id="backup-interval" class="w-full max-w-xs"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem value="24">每天</SelectItem><SelectItem value="168">每周</SelectItem></SelectGroup></SelectContent></Select></Field>
    <Field><FieldLabel for="backup-retain">保留最近几份自动备份</FieldLabel><Input id="backup-retain" class="max-w-xs" type="number" min="1" max="30" :model-value="store.settings.auto_backup.retain" @change="backupOptions({retain:Number(($event.target as HTMLInputElement).value)})"/><FieldDescription>1–30 份，仅清理旧的自动备份；手动备份和恢复前备份会保留。</FieldDescription></Field>
    <p class="text-sm text-muted-foreground">最近备份：{{ date(status?.last_backup??null) }} · {{ status?.count??0 }} 份</p><p class="break-all text-xs text-muted-foreground">{{ status?.directory }}</p><p v-if="status?.error" class="text-sm text-destructive">{{ status.error }}</p><FieldDescription>程序启动和运行期间每分钟检查一次；错过周期会在下次启动补备份。</FieldDescription><div class="flex flex-wrap gap-2"><Button variant="outline" :disabled="backupBusy" @click="backupNow">{{ backupBusy?'正在备份…':'立即备份' }}</Button><Button variant="outline" :disabled="backupBusy" @click="emit('restoreBackup')">选择备份恢复</Button></div></FieldGroup>
  </CardContent></Card>
  <Card v-if="props.section==='trash'||props.section==='all'"><CardHeader><div class="flex items-center justify-between gap-4"><CardTitle class="flex items-center gap-2"><Trash2 class="size-5 text-primary"/>回收站</CardTitle><Button variant="outline" size="sm" :disabled="!trash.length||trashBusy" @click="askPurge(null)">清空回收站</Button></div><CardDescription>删除的便签、笔记和倒数日保留在这里，可恢复原内容。不会自动清空。</CardDescription></CardHeader><CardContent>
    <p v-if="loading||error" class="text-sm text-muted-foreground">{{ error||'正在读取…' }}</p><p v-else-if="!trash.length" class="text-sm text-muted-foreground">回收站是空的。</p>
    <div class="flex max-h-96 flex-col gap-2 overflow-y-auto"><div v-for="item in trash" :key="item.id" class="flex items-center gap-3 rounded-xl bg-muted/50 p-3"><div class="min-w-0 flex-1"><p class="truncate text-sm font-medium">{{ item.title||'无标题' }}</p><p class="text-xs text-muted-foreground">{{ kindNames[item.kind] }} · 删除于 {{ date(item.deleted_at) }}</p></div><Button size="sm" variant="outline" :disabled="trashBusy" @click="restore(item)"><RotateCcw data-icon="inline-start"/>恢复</Button><Button variant="ghost" size="icon" :disabled="trashBusy" aria-label="永久删除" @click="askPurge(item.id)"><Trash2/></Button></div></div>
  </CardContent></Card>
  <ConfirmDialog v-model:open="purgeOpen" :title="purgeId?'永久删除这条内容？':'清空回收站？'" description="此操作无法撤销。需要的内容请先恢复，或创建备份。" destructive confirm-text="永久删除" @confirm="purge"/>
</template>
