<script setup lang="ts">
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { launcherOrder, launcherSections } from '@/lib/launcher-preferences'
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

import { onMounted, onUnmounted, ref, watch, computed } from 'vue'
import { Trash2, RotateCcw, GripVertical } from '@lucide/vue'
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
const shortcut=ref(store.settings.launcher.shortcut), shortcutError=ref(''), saving=ref(false), backupBusy=ref(false), notificationBusy=ref(false), notificationStatus=ref('')
const status=ref<BackupStatus>(), trash=ref<TrashItem[]>([]), loading=ref(true), error=ref(''), trashBusy=ref(false)
const purgeOpen=ref(false), purgeId=ref<string|null>(null)
const startupShortcutError=ref('')
let removeShortcutListener:UnlistenFn|undefined
const panelOrder=computed(()=>launcherOrder(store.settings.client))
const draggedPanel=ref<string|null>(null), draftOrder=ref<string[]|null>(null), orderSaving=ref(false)
const dragOffset=ref(0)
let dragStartY=0, dragStep=0
const dragOriginalOrder=ref<string[]>([])
const displayedOrder=computed(()=>draggedPanel.value?dragOriginalOrder.value:(draftOrder.value??panelOrder.value))
function panelDragStyle(id:string){
  if(!draggedPanel.value||!draftOrder.value)return undefined
  const offset=id===draggedPanel.value?dragOffset.value:(draftOrder.value.indexOf(id)-dragOriginalOrder.value.indexOf(id))*dragStep
  return {transform:`translateY(${offset}px)`}
}
function startPanelDrag(event:PointerEvent,id:string){
  if(event.button!==0||orderSaving.value)return
  event.preventDefault()
  const handle=event.currentTarget as HTMLElement
  const row=handle.closest<HTMLElement>('[data-panel-section]')
  if(!row)return
  const list=row.parentElement!
  const rows=[...list.querySelectorAll<HTMLElement>('[data-panel-section]')]
  dragStep=rows.length>1?rows[1]!.getBoundingClientRect().top-rows[0]!.getBoundingClientRect().top:row.offsetHeight
  dragStartY=event.clientY;dragOffset.value=0
  dragOriginalOrder.value=[...panelOrder.value]
  draggedPanel.value=id;draftOrder.value=[...panelOrder.value]
  handle.setPointerCapture(event.pointerId)
}
function movePanelDrag(event:PointerEvent){
  if(!draggedPanel.value||!draftOrder.value||!dragStep)return
  const original=dragOriginalOrder.value,from=original.indexOf(draggedPanel.value)
  dragOffset.value=Math.max(-from*dragStep,Math.min((original.length-1-from)*dragStep,event.clientY-dragStartY))
  const to=Math.max(0,Math.min(original.length-1,from+Math.round(dragOffset.value/dragStep)))
  const order=[...original]
  order.splice(from,1);order.splice(to,0,draggedPanel.value);draftOrder.value=order
}
async function finishPanelDrag(cancelled=false){
  if(!draggedPanel.value)return
  const order=draftOrder.value
  draggedPanel.value=null
  if(cancelled||!order||order.join()===panelOrder.value.join()){draftOrder.value=null;return}
  orderSaving.value=true
  try{await store.saveClient('lumo-launcher-order',order)}catch(e){store.notify('排序保存失败：'+String(e))}
  finally{draftOrder.value=null;orderSaving.value=false}
}
async function keyboardPanelMove(id:string,delta:number){
  if(orderSaving.value||draggedPanel.value)return
  const order=[...panelOrder.value],from=order.indexOf(id),to=from+delta
  if(to<0||to>=order.length)return
  order.splice(from,1);order.splice(to,0,id);orderSaving.value=true
  try{await store.saveClient('lumo-launcher-order',order)}catch(e){store.notify('排序保存失败：'+String(e))}finally{orderSaving.value=false}
}
const sizePresets=[{id:'default',label:'默认 · 1040 × 720'},{id:'640x480',label:'紧凑 · 640 × 480'},{id:'800x560',label:'标准 · 800 × 560'},{id:'1040x720',label:'宽敞 · 1040 × 720'},{id:'custom',label:'自定义'}]
const sizeChoice=ref('default'), panelWidth=ref<number|string>(1040), panelHeight=ref<number|string>(720), sizeError=ref('')
watch(()=>[store.settings.launcher.window_width,store.settings.launcher.window_height],([width,height])=>{
  panelWidth.value=width??1040;panelHeight.value=height??720
  sizeChoice.value=width&&height?(sizePresets.some(item=>item.id===`${width}x${height}`)?`${width}x${height}`:'custom'):'default'
},{immediate:true})
async function savePanelSize(){
  const width=Number(panelWidth.value),height=Number(panelHeight.value)
  if(sizeChoice.value!=='default'&&(!Number.isInteger(width)||width<480||width>3840||!Number.isInteger(height)||height<320||height>2160)){sizeError.value='宽度请输入 480–3840 的整数，高度请输入 320–2160 的整数';return}
  saving.value=true;sizeError.value=''
  try{await store.setSetting('launcher',{...store.settings.launcher,window_width:sizeChoice.value==='default'?null:width,window_height:sizeChoice.value==='default'?null:height});store.notify('窗口尺寸已保存，下次打开面板时生效')}
  catch(e){sizeError.value=String(e)}finally{saving.value=false}
}
function choosePanelSize(value:unknown){sizeChoice.value=String(value);sizeError.value='';if(sizeChoice.value==='custom')return;if(sizeChoice.value!=='default'){const [width,height]=sizeChoice.value.split('x').map(Number);panelWidth.value=width!;panelHeight.value=height!}void savePanelSize()}
const kindNames={note:'笔记',sticky:'便签',countdown:'倒数日'}
watch(()=>store.settings.launcher.shortcut,value=>shortcut.value=value)
watch(()=>store.settings,()=>void refresh())
let timer:ReturnType<typeof setInterval>
async function desktopOption(key:'close_to_tray'|'autostart',value:boolean){try{await store.setSetting('product',{...store.settings.product,[key]:value})}catch(e){store.notify('设置失败：'+String(e))}}
async function refresh(){try{const [s,t,e]=await Promise.all([api.autoBackupStatus(),api.listTrash(),api.launcherStatus()]);status.value=s;trash.value=t;startupShortcutError.value=e??'';error.value=''}catch(e){error.value=String(e)}finally{loading.value=false}}
onMounted(async()=>{removeShortcutListener=await listen<string|null>('lumo-shortcut-status',({payload})=>{startupShortcutError.value=payload??''});void refresh();timer=setInterval(refresh,30000)})
onUnmounted(()=>{clearInterval(timer);removeShortcutListener?.()})
async function configure(enabled=store.settings.launcher.enabled,minimalMode=store.settings.launcher.minimal_mode,saveShortcut=false){saving.value=true;try{store.settings=await api.configureLauncher({...store.settings.launcher,enabled,shortcut:saveShortcut?shortcut.value.trim():store.settings.launcher.shortcut,minimal_mode:minimalMode});shortcutError.value='';startupShortcutError.value='';store.notify('快捷启动设置已保存')}catch(e){shortcutError.value=String(e)}finally{saving.value=false}}
async function backupOptions(value:Partial<typeof store.settings.auto_backup>){try{await store.setSetting('auto_backup',{...store.settings.auto_backup,...value});await refresh()}catch{}}
async function backupNow(){backupBusy.value=true;try{await store.flushPendingSaves();await api.runAutoBackup();await refresh();store.notify('自动备份已创建')}catch(e){store.notify(String(e))}finally{backupBusy.value=false}}
async function testNotification(){notificationBusy.value=true;notificationStatus.value='';try{notificationStatus.value=await api.testReminderNotification()}catch(e){notificationStatus.value=String(e)}finally{notificationBusy.value=false}}
async function restore(item:TrashItem){trashBusy.value=true;try{await api.restoreTrash(item.id);await store.reload();await refresh();store.notify(`${kindNames[item.kind]}已恢复`)}catch(e){store.notify(String(e))}finally{trashBusy.value=false}}
function askPurge(id:string|null){purgeId.value=id;purgeOpen.value=true}
async function purge(){trashBusy.value=true;try{await api.purgeTrash(purgeId.value);await refresh();store.notify('已永久删除')}catch(e){store.notify(String(e))}finally{trashBusy.value=false}}
function date(value:string|null){return value?new Date(value).toLocaleString('zh-CN'):'尚未备份'}
</script>
<template>
  <div v-if="props.section==='launcher'||props.section==='all'" class="flex flex-col gap-6">
    <FieldGroup><Field><div><FieldLabel class="text-base font-semibold" for="close-to-tray">关闭窗口后留在托盘</FieldLabel><FieldDescription>快捷启动和日期提醒继续工作，可从托盘菜单退出。</FieldDescription></div><div class="flex items-center"><Switch id="close-to-tray" :model-value="store.settings.product.close_to_tray" @update:model-value="value=>desktopOption('close_to_tray',value)" /></div></Field>
    <Field><FieldLabel class="text-base font-semibold" for="autostart">开机启动</FieldLabel><div class="flex items-center"><Switch id="autostart" :model-value="store.settings.product.autostart" @update:model-value="value=>desktopOption('autostart',value)" /></div></Field>
    <Field><FieldLabel class="text-base font-semibold" for="reminder-hour">日期提醒时间</FieldLabel><Select :model-value="String(store.settings.product.reminder_hour)" @update:model-value="value=>store.setSetting('product',{...store.settings.product,reminder_hour:Number(value)}).catch(e=>store.notify(String(e)))"><SelectTrigger id="reminder-hour" class="w-40"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem v-for="hour in 24" :key="hour" :value="String(hour-1)">{{ String(hour-1).padStart(2,'0') }}:00</SelectItem></SelectGroup></SelectContent></Select><FieldDescription>按本机时区（{{ Intl.DateTimeFormat().resolvedOptions().timeZone||'本地时间' }}）检查。需要应用运行或留在托盘；关机期间无法提醒，重新启动后会补发最近错过的一天。</FieldDescription><div class="flex flex-wrap items-center gap-3"><Button variant="outline" :disabled="notificationBusy" @click="testNotification">{{ notificationBusy?'正在发送…':'发送测试通知' }}</Button><Button variant="ghost" @click="api.openNotificationSettings().catch(e=>store.notify(String(e)))">打开通知设置</Button><p v-if="notificationStatus" role="status" class="text-sm" :class="notificationStatus.includes('不可用')?'text-destructive':'text-muted-foreground'">{{ notificationStatus }}</p></div></Field>
    <Field><FieldLabel class="text-base font-semibold" for="launcher-enabled">启用全局快捷键</FieldLabel><div class="flex items-center"><Switch id="launcher-enabled" :model-value="store.settings.launcher.enabled" :disabled="saving" @update:model-value="value=>configure(value)"/></div></Field>
      <Field><FieldLabel class="text-base font-semibold" for="launcher-shortcut">快捷键</FieldLabel><div class="flex max-w-lg gap-2"><Input id="launcher-shortcut" v-model="shortcut" placeholder="Alt+Space" @keydown.enter="configure(store.settings.launcher.enabled,store.settings.launcher.minimal_mode,true)"/><Button :disabled="saving" @click="configure(store.settings.launcher.enabled,store.settings.launcher.minimal_mode,true)">保存</Button><Button variant="outline" @click="api.showLauncher().catch(e=>store.notify(String(e)))">预览面板</Button></div><FieldDescription v-if="shortcutError||startupShortcutError" class="text-destructive" role="alert">{{ shortcutError||startupShortcutError }}</FieldDescription><FieldDescription v-else>默认 Alt + Space，启动后自动生效。也可设置 Control+Alt+Space；修改后点击保存。面板支持 ↑ ↓、Enter 和 Esc。</FieldDescription></Field>
      <Field><div><FieldLabel class="text-base font-semibold" for="minimal-mode">极简模式</FieldLabel><FieldDescription>启动程序时直接显示搜索、最近内容和应用网格；点击主界面图标进入完整工作台。</FieldDescription></div><div class="flex items-center"><Switch id="minimal-mode" :model-value="store.settings.launcher.minimal_mode" :disabled="saving" @update:model-value="value=>configure(store.settings.launcher.enabled,value)"/></div></Field>
      <Field><FieldLabel class="text-base font-semibold">快捷启动面板顺序</FieldLabel><div class="flex max-w-lg flex-col gap-2" aria-label="快捷启动面板排序" :aria-busy="orderSaving"><div v-for="id in displayedOrder" :key="id" :data-panel-section="id" class="relative flex items-center gap-3 rounded-lg bg-muted/40 px-3 py-2" :style="panelDragStyle(id)" :class="draggedPanel===id?'z-10 bg-muted shadow-lg ring-2 ring-primary cursor-grabbing':'transition-transform duration-150 ease-out motion-reduce:transition-none'"><Button variant="ghost" size="icon-sm" class="touch-none cursor-grab active:cursor-grabbing" :disabled="orderSaving" :aria-label="`拖动排序：${launcherSections.find(item=>item.id===id)?.label}`" @pointerdown="startPanelDrag($event,id)" @pointermove="movePanelDrag" @pointerup="finishPanelDrag()" @pointercancel="finishPanelDrag(true)" @lostpointercapture="finishPanelDrag(true)" @keydown.up.prevent="keyboardPanelMove(id,-1)" @keydown.down.prevent="keyboardPanelMove(id,1)"><GripVertical /></Button><span class="flex-1 select-none text-sm">{{ launcherSections.find(item=>item.id===id)?.label }}</span></div></div><FieldDescription>拖动左侧手柄调整顺序，松开后自动保存；也可聚焦手柄后使用 ↑ ↓。已固定的自定义分组始终显示在全部应用上方。</FieldDescription></Field>
      <Field><FieldLabel class="text-base font-semibold" for="launcher-size">快捷启动窗口大小</FieldLabel><Select :model-value="sizeChoice" :disabled="saving" @update:model-value="choosePanelSize"><SelectTrigger id="launcher-size" class="w-full max-w-sm"><SelectValue/></SelectTrigger><SelectContent><SelectGroup><SelectItem v-for="preset in sizePresets" :key="preset.id" :value="preset.id">{{ preset.label }}</SelectItem></SelectGroup></SelectContent></Select><FieldDescription>单位为逻辑像素，适用于普通和极简模式；超出屏幕可用区域时自动缩小。默认尺寸为 1040 × 720。</FieldDescription></Field>
      <FieldGroup v-if="sizeChoice==='custom'" class="max-w-lg"><Field><FieldLabel class="text-base font-semibold" for="launcher-width">宽度</FieldLabel><Input id="launcher-width" v-model="panelWidth" type="number" min="480" max="3840" step="1" :aria-invalid="!!sizeError" @keydown.enter="savePanelSize"/></Field><Field><FieldLabel class="text-base font-semibold" for="launcher-height">高度</FieldLabel><Input id="launcher-height" v-model="panelHeight" type="number" min="320" max="2160" step="1" :aria-invalid="!!sizeError" @keydown.enter="savePanelSize"/></Field><div><Button :disabled="saving" @click="savePanelSize">保存窗口尺寸</Button></div></FieldGroup>
      <p v-if="sizeError" class="text-sm text-destructive" role="alert">{{ sizeError }}</p>
    </FieldGroup>
  </div>
  <div v-if="props.section==='backup'||props.section==='all'" class="flex flex-col gap-6">
    <FieldGroup><Field><FieldLabel class="text-base font-semibold" for="backup-enabled">启用定期备份</FieldLabel><div class="flex items-center"><Switch id="backup-enabled" :model-value="store.settings.auto_backup.enabled" @update:model-value="enabled=>backupOptions({enabled})"/></div></Field>
    <Field><FieldLabel class="text-base font-semibold" for="backup-interval">备份周期</FieldLabel><Select :model-value="String(store.settings.auto_backup.interval_hours)" @update:model-value="value=>backupOptions({interval_hours:Number(value)})"><SelectTrigger id="backup-interval" class="w-full max-w-xs"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem value="24">每天</SelectItem><SelectItem value="168">每周</SelectItem></SelectGroup></SelectContent></Select></Field>
    <Field><FieldLabel class="text-base font-semibold" for="backup-retain">保留最近几份自动备份</FieldLabel><Input id="backup-retain" class="max-w-xs" type="number" min="1" max="30" :model-value="store.settings.auto_backup.retain" @change="backupOptions({retain:Number(($event.target as HTMLInputElement).value)})"/><FieldDescription>1–30 份，仅清理旧的自动备份；手动备份和恢复前备份会保留。</FieldDescription></Field>
    <p class="text-sm text-muted-foreground">最近备份：{{ date(status?.last_backup??null) }} · {{ status?.count??0 }} 份</p><p class="break-all text-xs text-muted-foreground">{{ status?.directory }}</p><p v-if="status?.error" class="text-sm text-destructive">{{ status.error }}</p><FieldDescription>程序启动和运行期间每分钟检查一次；错过周期会在下次启动补备份。</FieldDescription><div class="flex flex-wrap gap-2"><Button variant="outline" :disabled="backupBusy" @click="backupNow">{{ backupBusy?'正在备份…':'立即备份' }}</Button><Button variant="outline" :disabled="backupBusy" @click="emit('restoreBackup')">选择备份恢复</Button></div></FieldGroup>
  </div>
  <div v-if="props.section==='trash'||props.section==='all'" class="flex flex-col gap-6">
    <div class="flex justify-end"><Button variant="outline" size="sm" :disabled="!trash.length||trashBusy" @click="askPurge(null)">清空回收站</Button></div>
    <p v-if="loading||error" class="text-sm text-muted-foreground">{{ error||'正在读取…' }}</p><p v-else-if="!trash.length" class="text-sm text-muted-foreground">回收站是空的。</p>
    <div class="flex flex-col gap-2"><div v-for="item in trash" :key="item.id" class="flex items-center gap-3 rounded-xl bg-muted/50 p-3"><div class="min-w-0 flex-1"><p class="truncate text-sm font-medium">{{ item.title||'无标题' }}</p><p class="text-xs text-muted-foreground">{{ kindNames[item.kind] }} · 删除于 {{ date(item.deleted_at) }}</p></div><Button size="sm" variant="outline" :disabled="trashBusy" @click="restore(item)"><RotateCcw data-icon="inline-start"/>恢复</Button><Button variant="danger" size="icon" :disabled="trashBusy" aria-label="永久删除" @click="askPurge(item.id)"><Trash2/></Button></div></div>
  </div>
  <ConfirmDialog v-model:open="purgeOpen" :title="purgeId?'永久删除这条内容？':'清空回收站？'" description="此操作无法撤销。需要的内容请先恢复，或创建备份。" destructive confirm-text="永久删除" @confirm="purge"/>
</template>
