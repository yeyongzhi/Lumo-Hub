<script setup lang="ts">
import QuickCapture from '@/components/QuickCapture.vue'
import { resourceImage } from '@/lib/library-preferences'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '@/components/ui/dialog'
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field'
import { countdownTarget } from '@/lib/dates'
import { format } from 'date-fns'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { CloudSun, Quote, Newspaper, Flower2, RefreshCw, AppWindow, ArrowUpRight, CalendarDays, Clock3, FileText, Globe, GripVertical, History, LayoutDashboard, Maximize2, ListFilter, NotebookPen, Plus, RotateCcw, StickyNote, Trash2 } from '@lucide/vue'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuLabel, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuCheckboxItem, DropdownMenuGroup, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { countdownStatus, formatLongDate } from '@/lib/dates'
import { useWorkspaceStore } from '@/stores/workspace'
import { defaultHomeLayout, homeTemplate, moveWidget, previewWidgetMove, widgetClasses, widgetNames, widgetSizes } from '@/lib/home-layout'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import HomeFeed from '@/components/HomeFeed.vue'
import type { FeedKind } from '@/lib/home-feeds'
import type { HomeWidget, HomeWidgetKind, HomeWidgetSize, PageId, Resource } from '@/lib/types'

const emit = defineEmits<{ navigate: [page: PageId] }>()
const store = useWorkspaceStore()
const captureOpen=ref(false), moduleSettings=ref<HomeWidget|null>(null), moduleSaving=ref(false)
const countdownCount=ref(String(store.settings.client?.['lumo-home-countdown-count']??3))
const layoutUndo=ref<HomeWidget[][]>([])
function rememberLayout(){layoutUndo.value.push(widgets.value.map(w=>({...w,selected_ids:[...(w.selected_ids??[])]})));if(layoutUndo.value.length>20)layoutUndo.value.shift()}
function undoLayout(){const previous=layoutUndo.value.pop();if(previous)widgets.value=previous}
function changeLayout(value:HomeWidget[]){rememberLayout();widgets.value=value}
function removeWidget(id:string){rememberLayout();widgets.value=widgets.value.filter(w=>w.id!==id)}
function configure(widget:HomeWidget){moduleSettings.value={...widget,selected_ids:[...(widget.selected_ids??[])]}}
async function saveModule(){if(!moduleSettings.value||moduleSaving.value)return;moduleSaving.value=true;try{await store.setSetting('home_layout',store.settings.home_layout.map(w=>w.id===moduleSettings.value!.id?moduleSettings.value!:w));await store.saveClient('lumo-home-countdown-count',Number(countdownCount.value));moduleSettings.value=null}catch(e){store.notify(String(e))}finally{moduleSaving.value=false}}
async function openContent(page:PageId,id?:string){try{await store.flushPendingSaves();if(id)store.searchTarget={id,kind:page==='notes'?'note':page==='stickies'?'sticky':'countdown'};emit('navigate',page)}catch(e){store.notify(String(e))}}
const now = ref(new Date())
let timer: ReturnType<typeof setInterval>
let unregisterSave: (() => void) | undefined
onMounted(() => { window.addEventListener('pointermove', pointerMove, { passive: false }); window.addEventListener('pointerup', pointerUp); window.addEventListener('pointercancel', cancelDrag); window.addEventListener('blur', cancelDrag); window.addEventListener('keydown', dragKeydown); timer = setInterval(() => { now.value = new Date() }, 1000); unregisterSave = store.registerPendingSave(async () => { if (editing.value) await persistLayout() }) })
onUnmounted(() => { stopAutoScroll(); window.removeEventListener('pointermove', pointerMove); window.removeEventListener('pointerup', pointerUp); window.removeEventListener('pointercancel', cancelDrag); window.removeEventListener('blur', cancelDrag); window.removeEventListener('keydown', dragKeydown); clearInterval(timer); unregisterSave?.() })
const longDate = computed(() => formatLongDate(now.value))
const time = computed(() => now.value.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' }))
const upcoming = computed(() => [...store.countdowns].filter(item=>!countdownStatus(item,now.value).past).sort((a, b) => countdownStatus(a, now.value).days - countdownStatus(b, now.value).days).slice(0, Number(store.settings.client?.['lumo-home-countdown-count']??3)))
function chosenSticky(widget: HomeWidget) { return widget.selected_ids?.length?store.stickies.find(item=>item.id===widget.selected_ids![0]):store.stickies.find(item=>store.settings.product.pinned_stickies.includes(item.id))||store.stickies[0] }
function chosenNote(widget: HomeWidget) { return widget.selected_ids?.length?store.notes.find(item=>item.id===widget.selected_ids![0]):store.notes[0] }
function chosenCountdowns(widget: HomeWidget) { return widget.selected_ids?.length?widget.selected_ids.map(id=>store.countdowns.find(item=>item.id===id)).filter((item):item is NonNullable<typeof item>=>!!item):upcoming.value }
function choices(widget:HomeWidget) { return widget.kind==='sticky'?store.stickies:widget.kind==='note'?store.notes:store.countdowns }
function selectContent(widget:HomeWidget,id:string) { const selected=widget.selected_ids??[]; widget.selected_ids=widget.kind==='countdowns'?(selected.includes(id)?selected.filter(x=>x!==id):[...selected,id]):[id] }
const lastSticky = computed(() => dragWidget.value?chosenSticky(dragWidget.value):store.stickies[0])
const lastNote = computed(() => dragWidget.value?chosenNote(dragWidget.value):store.notes[0])
const recentResources = computed(() => store.resources
  .filter(item => item.last_opened_at)
  .sort((a, b) => Date.parse(b.last_opened_at!) - Date.parse(a.last_opened_at!))
  .slice(0, 8))
const openingId = ref<string | null>(null)
const kindLabels = { app: '应用', web: '网页', file: '文件' } as const
const resourceIcons = { app: AppWindow, web: Globe, file: FileText }

function openedTime(value: string | null) {
  if (!value) return ''
  const date = new Date(value)
  const minutes = Math.max(0, Math.floor((now.value.getTime() - date.getTime()) / 60000))
  if (minutes < 1) return '刚刚打开'
  if (minutes < 60) return `${minutes} 分钟前`
  return date.toLocaleString('zh-CN', { month: 'numeric', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

async function launchRecent(item: Resource) {
  if (openingId.value) return
  openingId.value = item.id
  try { await store.launchResource(item.id) }
  catch (error) { store.notify(`无法打开“${item.name}”：${String(error)}。可在应用库中检查或重新定位路径。`) }
  finally { openingId.value = null }
}

const editing = ref(false)
const showModules = ref(false)
const libraryDragging = ref(false)
let libraryPointer:{kind:HomeWidgetKind;pointerId:number;x:number;y:number}|null=null
function startLibraryPointer(event:PointerEvent,kind:HomeWidgetKind){if(event.button!==0||saving.value||widgets.value.some(w=>w.kind===kind))return;event.preventDefault();libraryPointer={kind,pointerId:event.pointerId,x:event.clientX,y:event.clientY}}
const dropTarget = ref('')
function libraryDragOver(event:DragEvent){if(!libraryDragging.value)return;dropTarget.value=(event.target as HTMLElement).closest<HTMLElement>('[data-widget-id]')?.dataset.widgetId||''}
const newsDate = ref('')
const modulePreviews: Record<HomeWidgetKind,string> = { clock:'14:26:08 · 星期六', countdowns:'纪念日 · 还有 14 天', sticky:'随手记录想法和待办', note:'整理笔记，留住灵感', recent:'最近打开的应用和文件', weather:'22°C · 晴 · 体感 20°C', hitokoto:'一句话，遇见好心情。', news:'01 今日资讯\n02 每天一分钟，了解世界', poetry:'故人具鸡黍，邀我至田家。' }
function libraryDrag(event: DragEvent, kind: HomeWidgetKind) {
  if (saving.value || widgets.value.some(item=>item.kind===kind)) { event.preventDefault(); return }
  event.dataTransfer?.setData('application/x-lumo-module',kind)
  if(event.dataTransfer) event.dataTransfer.effectAllowed='copy'
  libraryDragging.value=true
}
function libraryDrop(event: DragEvent) {
  libraryDragging.value=false;dropTarget.value=''
  const kind=event.dataTransfer?.getData('application/x-lumo-module') as HomeWidgetKind
  if(!editing.value || saving.value || !Object.prototype.hasOwnProperty.call(widgetNames,kind) || widgets.value.some(item=>item.kind===kind))return
  const target=(event.target as HTMLElement).closest<HTMLElement>('[data-widget-id]')
  const index=target?widgets.value.findIndex(item=>item.id===target.dataset.widgetId):-1
  addModule(kind)
  if(index>=0){const item=widgets.value.pop()!;widgets.value.splice(index,0,item)}
}
const saving = ref(false)
const draggedId = ref<string | null>(null)
const dragPosition = ref({ x: 0, y: 0, offsetX: 0, offsetY: 0, width: 0, height: 0 })
let pendingDrag: { id: string; pointerId: number; startX: number; startY: number; original: HomeWidget[]; capture: HTMLElement | null } | null = null
let lastPreviewAt = 0
let autoScrollFrame = 0
let lastPointerEvent: PointerEvent | null = null
const dragWidget = computed(() => widgets.value.find(widget => widget.id === draggedId.value))
const ghostStyle = computed(() => ({ left: `${dragPosition.value.x - dragPosition.value.offsetX}px`, top: `${dragPosition.value.y - dragPosition.value.offsetY}px`, width: `${dragPosition.value.width}px`, height: `${dragPosition.value.height}px` }))
const dragPreviewText = computed(() => {
  switch (dragWidget.value?.kind) {
    case 'clock': return `${time.value} · ${longDate.value}`
    case 'countdowns': return (dragWidget.value?chosenCountdowns(dragWidget.value):upcoming.value).map(item => `${item.title} ${countdownStatus(item, now.value).text}`).join('\n') || '添加一个重要日期，让期待有个名字。'
    case 'sticky': return lastSticky.value?.content || '还没有便签，在这里记录待办或随手想法。'
    case 'note': return lastNote.value ? `${lastNote.value.title}\n${lastNote.value.content}` : '还没有笔记，开始记录和整理想法。'
    case 'recent': return recentResources.value.map(item => item.name).join(' · ') || '还没有打开记录。'
    default: return dragWidget.value ? modulePreviews[dragWidget.value.kind] : ''
  }
})
const widgets = ref<HomeWidget[]>(store.settings.home_layout.map(item => ({ ...item, selected_ids: [...(item.selected_ids??[])] })))
const moduleIcons = { clock: Clock3, countdowns: CalendarDays, sticky: StickyNote, note: NotebookPen, recent: History, weather: CloudSun, hitokoto: Quote, news: Newspaper, poetry: Flower2 }
function isOnline(kind:HomeWidgetKind):kind is FeedKind{return ['weather','hitokoto','news','poetry'].includes(kind)}
const feedUpdates=ref<Record<string,string>>({})
const feedLoading=ref<Record<string,boolean>>({})
const refreshKeys=ref<Record<FeedKind,number>>({weather:0,hitokoto:0,news:0,poetry:0})
const modulePages: Partial<Record<HomeWidgetKind, PageId>> = { countdowns: 'countdowns', sticky: 'stickies', note: 'notes', recent: 'library' } as const
watch(() => store.settings.home_layout, value => { if (!editing.value) widgets.value = value.map(item => ({ ...item, selected_ids: [...(item.selected_ids??[])] })) }, { deep: true })
function beginEdit() { layoutUndo.value=[];widgets.value = store.settings.home_layout.map(item => ({ ...item, selected_ids: [...(item.selected_ids??[])] })); editing.value = true }
function openModules() { if (!editing.value) beginEdit(); showModules.value = true }
function cancelEdit() { stopAutoScroll(); widgets.value = store.settings.home_layout.map(item => ({ ...item, selected_ids: [...(item.selected_ids??[])] })); editing.value = false; showModules.value = false; draggedId.value = null; pendingDrag = null }
function addModule(kind: HomeWidgetKind) { if (widgets.value.some(item => item.kind === kind)) return;rememberLayout(); widgets.value.push({ id: crypto.randomUUID(), kind, size: kind === 'news' ? '2x2' : kind === 'recent'||isOnline(kind) ? '2x1' : '1x1' }) }
function resize(widget: HomeWidget, size: HomeWidgetSize) { rememberLayout();widget.size = size }
function startDrag(event: PointerEvent, widget: HomeWidget) {
  if (!editing.value || saving.value || event.button !== 0) return
  const card = (event.currentTarget as HTMLElement).closest<HTMLElement>('[data-widget-id]')
  if (!card) return
  event.preventDefault()
  const rect = card.getBoundingClientRect()
  const capture = card.parentElement
  pendingDrag = { id: widget.id, pointerId: event.pointerId, startX: event.clientX, startY: event.clientY, original: widgets.value.map(item => ({ ...item, selected_ids: [...(item.selected_ids??[])] })), capture }
  capture?.setPointerCapture(event.pointerId)
  dragPosition.value = { x: event.clientX, y: event.clientY, offsetX: event.clientX - rect.left, offsetY: event.clientY - rect.top, width: rect.width, height: rect.height }
  lastPreviewAt = 0
}
function pointerMove(event: PointerEvent) {
  if(libraryPointer){libraryDragging.value=Math.hypot(event.clientX-libraryPointer.x,event.clientY-libraryPointer.y)>5;dropTarget.value=document.elementFromPoint(event.clientX,event.clientY)?.closest<HTMLElement>('[data-widget-id]')?.dataset.widgetId??'';return}
  if (!pendingDrag || event.pointerId !== pendingDrag.pointerId) return
  if (!draggedId.value && Math.hypot(event.clientX - pendingDrag.startX, event.clientY - pendingDrag.startY) < 5) return
  event.preventDefault()
  const started = !draggedId.value
  draggedId.value = pendingDrag.id
  lastPointerEvent = event
  if (started) autoScrollFrame = requestAnimationFrame(autoScroll)
  dragPosition.value.x = event.clientX
  dragPosition.value.y = event.clientY
  if (performance.now() - lastPreviewAt < 160) return
  let target = document.elementsFromPoint(event.clientX, event.clientY)
    .map(element => element.closest<HTMLElement>('[data-widget-id]'))
    .find(element => element && element.dataset.widgetId !== draggedId.value)
  if (!target) {
    const grid = document.querySelector<HTMLElement>('[data-home-grid]')
    const bounds = grid?.getBoundingClientRect()
    if (!grid || !bounds || event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) return
    // Keep the current placeholder stable while the pointer is inside it.
    if (document.elementsFromPoint(event.clientX,event.clientY).some(element => element.closest<HTMLElement>('[data-widget-id]')?.dataset.widgetId === draggedId.value)) return
    target = [...grid.querySelectorAll<HTMLElement>('[data-widget-id]')].filter(element => element.dataset.widgetId !== draggedId.value)
      .sort((a,b) => distance(a,event) - distance(b,event))[0]
  }
  if (!target) return
  const rect = target.getBoundingClientRect()
  const source = [...document.querySelectorAll<HTMLElement>('[data-widget-id]')].find(element => element.dataset.widgetId === draggedId.value)?.getBoundingClientRect()
  const sameRow = source && Math.abs(source.top - rect.top) < Math.min(source.height,rect.height) / 2
  const delta = sameRow ? event.clientX - (rect.left + rect.width / 2) : event.clientY - (rect.top + rect.height / 2)
  if (Math.abs(delta) < 8) return
  dropTarget.value=target.dataset.widgetId||''
  const next = previewWidgetMove(widgets.value, draggedId.value, target.dataset.widgetId!, delta > 0)
  if (next !== widgets.value) { widgets.value = next; lastPreviewAt = performance.now() }
}
function distance(element: HTMLElement, event: PointerEvent) { const rect=element.getBoundingClientRect(); return Math.hypot(event.clientX-Math.max(rect.left,Math.min(rect.right,event.clientX)),event.clientY-Math.max(rect.top,Math.min(rect.bottom,event.clientY))) }
function stopAutoScroll() { dropTarget.value=''; cancelAnimationFrame(autoScrollFrame); autoScrollFrame=0; lastPointerEvent=null }
function autoScroll() {
  if (!draggedId.value || !lastPointerEvent) return
  const container=document.querySelector<HTMLElement>('[data-home-grid]')?.closest('main')
  if (container) {
    const rect=container.getBoundingClientRect()
    const y=dragPosition.value.y
    const delta=y<rect.top+48?-10:y>rect.bottom-48?10:0
    if(delta){const before=container.scrollTop;container.scrollTop+=delta;if(container.scrollTop!==before)pointerMove(lastPointerEvent)}
  }
  autoScrollFrame=requestAnimationFrame(autoScroll)
}
function releasePointer() { if(pendingDrag?.capture?.hasPointerCapture(pendingDrag.pointerId))pendingDrag.capture.releasePointerCapture(pendingDrag.pointerId) }
function pointerUp(event: PointerEvent) { if(libraryPointer?.pointerId===event.pointerId){const kind=libraryPointer.kind;const target=dropTarget.value;libraryPointer=null;libraryDragging.value=false;dropTarget.value='';const index=widgets.value.findIndex(w=>w.id===target);addModule(kind);if(index>=0){const widget=widgets.value.pop()!;widgets.value.splice(index,0,widget)}return} if (pendingDrag?.pointerId===event.pointerId) { layoutUndo.value.push(pendingDrag.original);stopAutoScroll(); releasePointer(); pendingDrag=null; draggedId.value=null } }
function cancelDrag() { libraryPointer=null;libraryDragging.value=false; stopAutoScroll(); if (pendingDrag && draggedId.value) widgets.value=pendingDrag.original; releasePointer(); pendingDrag=null; draggedId.value=null }
function dragKeydown(event: KeyboardEvent) { if (event.key==='Escape' && pendingDrag) { event.preventDefault();cancelDrag() } }
function moveByKeyboard(event: KeyboardEvent, widget: HomeWidget) { const delta=['ArrowLeft','ArrowUp'].includes(event.key)?-1:['ArrowRight','ArrowDown'].includes(event.key)?1:0; if(delta&&!saving.value){event.preventDefault();rememberLayout();const index=widgets.value.findIndex(item=>item.id===widget.id);widgets.value=moveWidget(widgets.value,index,index+delta)} }
async function persistLayout() { saving.value = true; try { await store.setSetting('home_layout', widgets.value.map(item => ({ ...item, selected_ids: [...(item.selected_ids??[])] }))); editing.value = false; showModules.value = false; store.notify('首页布局已保存') } finally { saving.value = false } }
async function saveLayout() { try { await persistLayout() } catch {} }
</script>

<template>
  <div class="@container flex w-full flex-col gap-4 pb-10">
    <Dialog v-model:open="captureOpen"><DialogContent><DialogHeader><DialogTitle>快速记录</DialogTitle><DialogDescription>保存想法、笔记或重要日期。</DialogDescription></DialogHeader><QuickCapture @saved="captureOpen=false"/></DialogContent></Dialog>
    <Dialog :open="!!moduleSettings" @update:open="value=>{if(!value)moduleSettings=null}"><DialogContent><DialogHeader><DialogTitle>设置卡片内容</DialogTitle><DialogDescription>选择首页展示的记录，修改后保存生效。</DialogDescription></DialogHeader><FieldGroup v-if="moduleSettings"><Field><FieldLabel>显示内容</FieldLabel><Button variant="outline" @click="moduleSettings.selected_ids=[]">自动显示最近内容</Button><div class="max-h-64 overflow-y-auto"><Button v-for="item in choices(moduleSettings)" :key="item.id" variant="ghost" class="w-full justify-start" :aria-pressed="moduleSettings.selected_ids?.includes(item.id)" @click="selectContent(moduleSettings,item.id)">{{ moduleSettings.selected_ids?.includes(item.id)?'✓ ':'' }}{{ item.title||'无标题' }}</Button></div></Field><Field v-if="moduleSettings.kind==='countdowns'"><FieldLabel>自动显示数量</FieldLabel><Select v-model="countdownCount"><SelectTrigger><SelectValue/></SelectTrigger><SelectContent><SelectGroup><SelectItem v-for="count in [3,5,8]" :key="count" :value="String(count)">{{ count }} 个日期</SelectItem></SelectGroup></SelectContent></Select></Field></FieldGroup><Button :disabled="moduleSaving" @click="saveModule">{{ moduleSaving?'保存中…':'保存卡片设置' }}</Button></DialogContent></Dialog>
    <section class="flex flex-wrap items-center justify-end gap-4"><Button v-if="!editing" variant="outline" class="mr-auto" @click="captureOpen=true"><Plus data-icon="inline-start"/>快速记录</Button>
      <Button v-if="!editing" variant="outline" @click="beginEdit"><LayoutDashboard data-icon="inline-start" />编辑首页</Button>
      <div v-else class="flex flex-wrap gap-2">
        <Button variant="outline" :disabled="saving" @click="showModules=!showModules"><Plus data-icon="inline-start" />打开卡片库</Button>
        <DropdownMenu><DropdownMenuTrigger as-child><Button variant="outline" :disabled="saving">选择布局模板</Button></DropdownMenuTrigger><DropdownMenuContent align="end"><DropdownMenuLabel>选择后预览，保存才生效</DropdownMenuLabel><DropdownMenuGroup><DropdownMenuItem @select="changeLayout(homeTemplate('simple'))">简洁 · 时间、最近打开、便签</DropdownMenuItem><DropdownMenuItem @select="changeLayout(homeTemplate('work'))">工作 · 启动、笔记、便签、日期</DropdownMenuItem><DropdownMenuItem @select="changeLayout(homeTemplate('life'))">生活 · 日期、天气、每日内容</DropdownMenuItem></DropdownMenuGroup></DropdownMenuContent></DropdownMenu>
        <ConfirmDialog title="恢复默认首页布局？" description="只恢复模块位置和尺寸，便签、笔记和应用库数据会保留。" confirm-text="恢复默认" @confirm="changeLayout(defaultHomeLayout())"><template #trigger><Button variant="ghost" :disabled="saving"><RotateCcw data-icon="inline-start" />恢复默认</Button></template></ConfirmDialog>
        <Button variant="outline" :disabled="saving||!layoutUndo.length" @click="undoLayout">撤销调整</Button><Button variant="ghost" :disabled="saving" @click="cancelEdit">取消</Button><Button :disabled="saving" @click="saveLayout">{{ saving?'保存中…':'保存布局' }}</Button>
      </div>
    </section>
    <Card v-if="editing&&showModules">
      <CardHeader><CardTitle>卡片库</CardTitle><CardDescription>拖动预览卡片到下方区域即可添加，每种卡片可添加一个。</CardDescription></CardHeader>
      <CardContent class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-5">
        <div v-for="(name,kind) in widgetNames" :key="kind" :draggable="false" @pointerdown="startLibraryPointer($event,kind)" tabindex="0" role="button" @keydown.enter.prevent="addModule(kind)" @keydown.space.prevent="addModule(kind)" :aria-disabled="saving||widgets.some(item=>item.kind===kind)" class="flex min-h-28 flex-col gap-3 rounded-xl border bg-background p-3" :class="widgets.some(item=>item.kind===kind)?'opacity-45':'cursor-grab hover:border-primary active:cursor-grabbing'" @dragstart="libraryDrag($event,kind)" @dragend="libraryDragging=false;dropTarget=''">
          <div class="flex items-center gap-2 text-sm font-medium"><component :is="moduleIcons[kind]" class="size-4 text-primary" />{{ name }}<span v-if="widgets.some(item=>item.kind===kind)" class="ml-auto text-xs text-muted-foreground">已添加</span></div>
          <p class="whitespace-pre-line text-sm leading-6 text-muted-foreground">{{ modulePreviews[kind] }}</p>
        </div>
      </CardContent>
    </Card>
    <p v-if="editing" class="text-sm text-muted-foreground">拖动左上角手柄实时调整位置，点击右上角尺寸图标选择宽 × 高，内容图标可指定便签、笔记或倒数日。窄窗口会自动调整为两列。</p>
    <TransitionGroup v-if="widgets.length" tag="div" data-home-grid @dragover.prevent="libraryDragOver" @drop.prevent="libraryDrop" :class="libraryDragging?'rounded-xl ring-2 ring-primary/40':''" move-class="transition-transform duration-150 ease-out motion-reduce:transition-none" class="grid grid-cols-2 auto-rows-[12rem] gap-4 @min-[64rem]:grid-cols-4">
      <Card v-for="widget in widgets" :key="widget.id" :data-widget-id="widget.id" class="relative min-h-0 min-w-0 gap-0 overflow-hidden py-0" :class="[widgetClasses[widget.size],draggedId===widget.id?'opacity-30 ring-2 ring-primary ring-offset-2 ring-offset-background':'']">
        <div v-if="editing&&dropTarget===widget.id" class="pointer-events-none absolute inset-x-0 top-0 z-10 border-t-4 border-primary"><span class="rounded-b bg-primary px-2 py-1 text-xs text-primary-foreground">{{ libraryDragging?'添加到这里':'移到这里' }}</span></div>
        <div v-if="editing" class="flex shrink-0 items-center justify-between gap-1 border-b border-border bg-muted/50 px-3 py-2">
          <Button variant="ghost" size="icon-sm" class="touch-none cursor-grab active:cursor-grabbing" :disabled="saving" :aria-label="`拖动${widgetNames[widget.kind]}`" @pointerdown="startDrag($event,widget)" @keydown="moveByKeyboard($event,widget)"><GripVertical /></Button>
          <span class="truncate text-xs text-muted-foreground">{{ widgetNames[widget.kind] }}</span>
          <div class="flex shrink-0 gap-1">
            <DropdownMenu v-if="['sticky','note','countdowns'].includes(widget.kind)"><DropdownMenuTrigger as-child><Button variant="ghost" size="icon-xs" aria-label="选择模块内容" title="选择显示内容"><ListFilter/></Button></DropdownMenuTrigger><DropdownMenuContent align="end" class="max-h-80 w-60 overflow-y-auto"><DropdownMenuLabel>选择显示的内容</DropdownMenuLabel><DropdownMenuGroup><DropdownMenuItem @select="widget.selected_ids=[]">自动显示{{ widget.kind==='countdowns'?'最近日期':'最新内容' }}</DropdownMenuItem></DropdownMenuGroup><DropdownMenuGroup><DropdownMenuCheckboxItem v-for="item in choices(widget)" :key="item.id" :model-value="widget.selected_ids?.includes(item.id)??false" @select="$event.preventDefault()" @update:model-value="selectContent(widget,item.id)">{{ item.title||'无标题' }}</DropdownMenuCheckboxItem></DropdownMenuGroup><DropdownMenuLabel v-if="!choices(widget).length">暂无内容，请先创建</DropdownMenuLabel></DropdownMenuContent></DropdownMenu>
            <DropdownMenu>
              <DropdownMenuTrigger as-child><Button variant="ghost" size="icon-xs" :disabled="saving" :aria-label="`调整${widgetNames[widget.kind]}尺寸，当前${widget.size.replace('x','乘')}`" title="调整布局尺寸"><Maximize2 /></Button></DropdownMenuTrigger>
              <DropdownMenuContent align="end"><DropdownMenuLabel>布局尺寸 · 宽 × 高</DropdownMenuLabel><DropdownMenuRadioGroup :model-value="widget.size" @update:model-value="value=>resize(widget,value as HomeWidgetSize)"><DropdownMenuRadioItem v-for="size in widgetSizes" :key="size" :value="size">{{ size.replace('x','×') }}</DropdownMenuRadioItem></DropdownMenuRadioGroup></DropdownMenuContent>
            </DropdownMenu>
            <Button variant="danger" size="icon-xs" :disabled="saving" :aria-label="`移除${widgetNames[widget.kind]}模块`" @click="removeWidget(widget.id)"><Trash2 /></Button>
          </div>
        </div>
        <div class="flex shrink-0 items-center justify-between gap-2 px-4 pt-3 pb-2">
          <div class="flex min-w-0 items-center gap-3"><span class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary"><component :is="moduleIcons[widget.kind]" class="size-5" /></span><h2 class="shrink-0 font-semibold">{{ widgetNames[widget.kind] }}</h2><span v-if="['weather','hitokoto','poetry'].includes(widget.kind)&&feedUpdates[widget.id]" class="truncate text-xs text-muted-foreground">{{ feedUpdates[widget.id] }}</span><span v-if="widget.kind==='news'&&newsDate" class="truncate text-sm text-muted-foreground">{{ newsDate }} · 每天一分钟，了解世界</span></div>
          <Button v-if="!editing&&['sticky','note','countdowns'].includes(widget.kind)" variant="ghost" size="icon-sm" aria-label="设置卡片内容" @click="configure(widget)"><ListFilter/></Button><Button v-if="isOnline(widget.kind)" variant="ghost" size="icon-sm" :disabled="feedLoading[widget.id]" :aria-label="feedLoading[widget.id]?`正在刷新${widgetNames[widget.kind]}`:`刷新${widgetNames[widget.kind]}`" :title="feedLoading[widget.id]?`正在刷新${widgetNames[widget.kind]}`:`刷新${widgetNames[widget.kind]}`" @click="refreshKeys[widget.kind]++"><RefreshCw :class="feedLoading[widget.id]?'animate-spin':''" /></Button><Button v-else-if="modulePages[widget.kind]" variant="ghost" size="icon-sm" :aria-label="`打开${widgetNames[widget.kind]}`" @click="emit('navigate',modulePages[widget.kind]!)"><ArrowUpRight /></Button>
        </div>
        <div class="min-h-0 flex-1 overflow-hidden px-4 pb-5"><div class="flex h-full min-h-0 flex-col gap-2 overflow-y-auto pr-1">
          <HomeFeed v-if="isOnline(widget.kind)" :kind="widget.kind" :refresh-key="refreshKeys[widget.kind]" @updated="value=>feedUpdates[widget.id]=value" @metadata="value=>{if(widget.kind==='news')newsDate=value}" @loading="value=>feedLoading[widget.id]=value" /><template v-else-if="widget.kind==='clock'"><div class="flex flex-1 flex-col justify-center gap-2"><p class="text-4xl font-semibold tabular-nums tracking-tight">{{ time }}</p><p class="text-sm text-muted-foreground">{{ longDate }}</p></div></template>
          <template v-else-if="widget.kind==='countdowns'"><Button variant="ghost" :disabled="editing" @click="openContent('countdowns',item.id)" v-for="item in chosenCountdowns(widget)" :key="item.id" class="h-auto w-full items-center justify-between gap-3 rounded-lg bg-muted/40 px-3 py-2 text-left"><div class="min-w-0"><p class="truncate text-sm font-medium">{{ item.title }}</p><p class="mt-0.5 text-xs text-muted-foreground">{{ format(countdownTarget(item,now),'yyyy-MM-dd') }}</p></div><span class="shrink-0 text-sm font-semibold text-primary">{{ countdownStatus(item,now).text }}</span></Button><p v-if="!chosenCountdowns(widget).length" class="text-sm leading-6 text-muted-foreground">{{ widget.selected_ids?.length?'选中的日期已删除，请重新选择。':'添加一个重要日期，让期待有个名字。' }}</p></template>
          <template v-else-if="widget.kind==='sticky'"><Button v-if="chosenSticky(widget)" variant="ghost" :disabled="editing" class="h-auto w-full justify-start p-0 text-left" @click="openContent('stickies',chosenSticky(widget)?.id)"><span class="truncate">{{ chosenSticky(widget)?.title||'无标题便签' }}</span></Button><p class="line-clamp-4 whitespace-pre-wrap break-words text-sm leading-6 text-muted-foreground">{{ chosenSticky(widget)?.content||(widget.selected_ids?.length?'选中的便签已删除，请重新选择。':'留一块空白，给灵感落脚。') }}</p></template>
          <template v-else-if="widget.kind==='note'"><Button v-if="chosenNote(widget)" variant="ghost" :disabled="editing" class="h-auto w-full justify-start p-0 text-left" @click="openContent('notes',chosenNote(widget)?.id)"><span class="truncate">{{ chosenNote(widget)?.title||'无标题笔记' }}</span></Button><p class="line-clamp-4 whitespace-pre-wrap break-words text-sm leading-6 text-muted-foreground">{{ chosenNote(widget)?chosenNote(widget)?.content||'这篇笔记还没有内容。':widget.selected_ids?.length?'选中的笔记已删除，请重新选择。':'创建第一篇笔记，开始整理想法。' }}</p></template>
          <template v-else>
            <div v-if="recentResources.length" class="grid grid-cols-[repeat(auto-fit,minmax(min(100%,11rem),1fr))] gap-2">
              <Button v-for="item in recentResources" :key="item.id" variant="outline" class="h-auto min-w-0 justify-start gap-2 p-3" :disabled="openingId===item.id" :aria-label="`打开 ${item.name}`" :title="item.target" @click="launchRecent(item)"><img v-if="resourceImage(store.settings.client,item)" :src="resourceImage(store.settings.client,item)!" alt="" class="size-7 shrink-0 object-contain" /><component :is="resourceIcons[item.kind]" v-else data-icon="inline-start" /><span class="flex min-w-0 flex-1 flex-col gap-1 text-left"><span class="truncate">{{ openingId===item.id?'正在打开…':item.name }}</span><span class="text-xs text-muted-foreground">{{ kindLabels[item.kind] }} · {{ openedTime(item.last_opened_at) }}</span></span></Button>
            </div><p v-else class="my-auto text-sm text-muted-foreground">还没有打开记录。从应用库打开项目后，会自动显示在这里。</p>
          </template>
        </div></div>
      </Card>
    </TransitionGroup>
    <Card v-else @dragover.prevent="libraryDragOver" @drop.prevent="libraryDrop"><CardHeader><CardTitle>首页还没有模块</CardTitle><CardDescription>添加日期时间、天气、新闻、诗词和记录模块，组合你的首页。</CardDescription></CardHeader><CardContent><Button :disabled="saving" @click="openModules"><Plus data-icon="inline-start" />打开卡片库</Button></CardContent></Card>
    <Teleport to="body">
      <Card v-if="dragWidget" :style="ghostStyle" class="pointer-events-none fixed z-50 select-none gap-4 overflow-hidden border-primary bg-card/95 p-4 shadow-xl">
        <div class="flex items-center gap-3"><component :is="moduleIcons[dragWidget.kind]" class="size-5 text-primary" /><h2 class="font-semibold">{{ widgetNames[dragWidget.kind] }}</h2><span class="ml-auto text-xs text-muted-foreground">{{ dragWidget.size.replace('x','×') }}</span></div>
        <p class="line-clamp-6 whitespace-pre-wrap text-sm leading-6 text-muted-foreground">{{ dragPreviewText }}</p>
      </Card>
    </Teleport>
  </div>
</template>
