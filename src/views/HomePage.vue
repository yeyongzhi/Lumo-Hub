<script setup lang="ts">
import QuickCapture from '@/components/QuickCapture.vue'
import { builtinTools, openTool } from '@/lib/tools'
import { resourceImage } from '@/lib/library-preferences'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '@/components/ui/dialog'
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field'
import { countdownTarget } from '@/lib/dates'
import { format } from 'date-fns'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { CloudSun, Quote, Newspaper, Flower2, RefreshCw, AppWindow, ArrowUpRight, CalendarDays, Clock3, FileText, Globe, GripVertical, History, LayoutDashboard, ListFilter, NotebookPen, Plus, RotateCcw, StickyNote, Trash2, Grid2X2, Rocket, Wrench, ChevronLeft, ChevronRight } from '@lucide/vue'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { DropdownMenu, DropdownMenuContent, DropdownMenuLabel, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuCheckboxItem, DropdownMenuGroup, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { countdownStatus, formatLongDate } from '@/lib/dates'
import { useWorkspaceStore } from '@/stores/workspace'
import { defaultHomeLayout, homeTemplate, moveWidget, previewWidgetMove, normalizeHomeWidget, fixedWidgetSize, widgetClasses, widgetNames, widgetSizes } from '@/lib/home-layout'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import HomeFeed from '@/components/HomeFeed.vue'
import type { FeedKind } from '@/lib/home-feeds'
import type { HomeWidget, HomeWidgetKind, HomeWidgetSize, PageId, Resource } from '@/lib/types'

const emit = defineEmits<{ navigate: [page: PageId] }>()
const store = useWorkspaceStore()
const captureOpen=ref(false), moduleSettings=ref<HomeWidget|null>(null), moduleSaving=ref(false)
const contentAuto=ref(true)
const countdownCount=ref(String(store.settings.client?.['lumo-home-countdown-count']??3))
const layoutUndo=ref<HomeWidget[][]>([])
type SavedHomeTemplate = { id:string; name:string; widgets:HomeWidget[] }
const layoutTemplatesKey='lumo-home-layout-templates'
const savedTemplates=ref<SavedHomeTemplate[]>(Array.isArray(store.settings.client?.[layoutTemplatesKey]) ? store.settings.client![layoutTemplatesKey] as SavedHomeTemplate[] : [])
const layoutDialog=ref(false), selectedTemplate=ref(''), templateName=ref(''), templateBusy=ref(false)
const presetTemplates=[{id:'simple',name:'简洁',description:'时间、最近打开、便签'},{id:'work',name:'工作',description:'启动、笔记、便签、日期'},{id:'life',name:'生活',description:'日期、天气、每日内容'}] as const
function openLayoutTemplates(){selectedTemplate.value='';templateName.value='';layoutDialog.value=true}
async function saveCurrentTemplate(){const name=templateName.value.trim();if(!name||templateBusy.value)return;const next=[...savedTemplates.value,{id:crypto.randomUUID(),name,widgets:widgets.value.map(normalizeHomeWidget)}];templateBusy.value=true;try{await store.saveClient(layoutTemplatesKey,next);savedTemplates.value=next;selectedTemplate.value=next[next.length-1].id;templateName.value='';store.notify('布局模板已保存')}catch(error){store.notify(`保存布局模板失败：${String(error)}`)}finally{templateBusy.value=false}}
async function deleteTemplate(id:string){if(templateBusy.value)return;const next=savedTemplates.value.filter(item=>item.id!==id);templateBusy.value=true;try{await store.saveClient(layoutTemplatesKey,next);savedTemplates.value=next;if(selectedTemplate.value===id)selectedTemplate.value=''}catch(error){store.notify(`删除布局模板失败：${String(error)}`)}finally{templateBusy.value=false}}
function applyTemplate(){const preset=presetTemplates.find(item=>`preset:${item.id}`===selectedTemplate.value);const saved=savedTemplates.value.find(item=>item.id===selectedTemplate.value);if(preset)changeLayout(homeTemplate(preset.id));else if(saved)changeLayout(saved.widgets.map(normalizeHomeWidget));else return;layoutDialog.value=false}

function rememberLayout(){layoutUndo.value.push(widgets.value.map(w=>({...w,selected_ids:[...(w.selected_ids??[])]})));if(layoutUndo.value.length>20)layoutUndo.value.shift()}
function changeLayout(value:HomeWidget[]){rememberLayout();widgets.value=value}
function removeWidget(id:string){rememberLayout();widgets.value=widgets.value.filter(w=>w.id!==id)}
function configure(widget:HomeWidget){moduleSettings.value={...widget,selected_ids:[...(widget.selected_ids??[])]};contentAuto.value=!widget.selected_ids?.length}
async function saveModule(){if(!moduleSettings.value||moduleSaving.value)return;if(['quick_launch','quick_tools'].includes(moduleSettings.value.kind)){if(!contentAuto.value&&!moduleSettings.value.selected_ids?.length)return;if(contentAuto.value)moduleSettings.value.selected_ids=[]}if(['sticky','note'].includes(moduleSettings.value.kind)){if(!contentAuto.value&&(moduleSettings.value.selected_ids?.length??0)!==2)return;if(contentAuto.value)moduleSettings.value.selected_ids=[]}moduleSaving.value=true;try{await store.setSetting('home_layout',store.settings.home_layout.map(w=>normalizeHomeWidget(w.id===moduleSettings.value!.id?moduleSettings.value!:w)));await store.saveClient('lumo-home-countdown-count',Number(countdownCount.value));moduleSettings.value=null}catch(e){store.notify(String(e))}finally{moduleSaving.value=false}}
async function openContent(page:PageId,id?:string){try{await store.flushPendingSaves();if(id)store.searchTarget={id,kind:page==='notes'?'note':page==='stickies'?'sticky':'countdown'};emit('navigate',page)}catch(e){store.notify(String(e))}}
const now = ref(new Date())
let timer: ReturnType<typeof setInterval>
let unregisterSave: (() => void) | undefined
onMounted(() => { window.addEventListener('pointermove', pointerMove, { passive: false }); window.addEventListener('pointerup', pointerUp); window.addEventListener('pointercancel', cancelDrag); window.addEventListener('blur', cancelDrag); window.addEventListener('keydown', dragKeydown); timer = setInterval(() => { now.value = new Date() }, 1000); unregisterSave = store.registerPendingSave(async () => { if (editing.value) await persistLayout() }) })
onUnmounted(() => { stopAutoScroll(); window.removeEventListener('pointermove', pointerMove); window.removeEventListener('pointerup', pointerUp); window.removeEventListener('pointercancel', cancelDrag); window.removeEventListener('blur', cancelDrag); window.removeEventListener('keydown', dragKeydown); clearInterval(timer); unregisterSave?.() })
const longDate = computed(() => formatLongDate(now.value))
const time = computed(() => now.value.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' }))
const calendarView = ref(new Date(now.value.getFullYear(), now.value.getMonth(), 1))
const calendarMonth = computed(() => `${calendarView.value.getFullYear()} 年 ${calendarView.value.getMonth()+1} 月`)
const calendarIsCurrent = computed(() => calendarView.value.getFullYear()===now.value.getFullYear() && calendarView.value.getMonth()===now.value.getMonth())
const clockCalendarView = ref(new Date(now.value.getFullYear(),now.value.getMonth(),1))
const clockCalendarMonth = computed(() => `${clockCalendarView.value.getFullYear()} 年 ${clockCalendarView.value.getMonth()+1} 月`)
const clockCalendarIsCurrent = computed(() => clockCalendarView.value.getFullYear()===now.value.getFullYear() && clockCalendarView.value.getMonth()===now.value.getMonth())
function moveClockCalendar(delta:number) { clockCalendarView.value=new Date(clockCalendarView.value.getFullYear(),clockCalendarView.value.getMonth()+delta,1) }
function clockCalendarToday() { clockCalendarView.value=new Date(now.value.getFullYear(),now.value.getMonth(),1) }
const currentMonthDays = computed(() => {
  const year=clockCalendarView.value.getFullYear(), month=clockCalendarView.value.getMonth()
  const offset=(new Date(year,month,1).getDay()+6)%7, days=new Date(year,month+1,0).getDate()
  return Array.from({length:Math.ceil((offset+days)/7)*7},(_,index)=>{const day=index-offset+1;return day>0&&day<=days?day:null})
})
function moveCalendar(delta:number) { calendarView.value=new Date(calendarView.value.getFullYear(),calendarView.value.getMonth()+delta,1) }
function calendarToday() { calendarView.value=new Date(now.value.getFullYear(),now.value.getMonth(),1) }
const calendarDays = computed(() => {
  const year=calendarView.value.getFullYear(), month=calendarView.value.getMonth()
  const offset=(new Date(year,month,1).getDay()+6)%7
  const days=new Date(year,month+1,0).getDate()
  return Array.from({length:Math.ceil((offset+days)/7)*7},(_,index)=>{
    const day=index-offset+1
    return day>0&&day<=days?day:null
  })
})
const upcoming = computed(() => {
  const items = [...store.countdowns]
  const future = items.filter(item => !countdownStatus(item, now.value).past).sort((a, b) => countdownStatus(a, now.value).days - countdownStatus(b, now.value).days)
  const past = items.filter(item => countdownStatus(item, now.value).past).sort((a, b) => countdownTarget(b, now.value).getTime() - countdownTarget(a, now.value).getTime())
  return [...future, ...past].slice(0, Number(store.settings.client?.['lumo-home-countdown-count'] ?? 3))
})
function chosenSticky(widget: HomeWidget) { return widget.selected_ids?.length?store.stickies.find(item=>item.id===widget.selected_ids![0]):store.stickies[0] }
function visibleStickies(widget: HomeWidget) {
  if (widget.selected_ids?.length) return store.stickies.filter(item => widget.selected_ids!.includes(item.id))
  return store.stickies.slice(0, 2)
}
function visibleNotes(widget: HomeWidget) { return widget.selected_ids?.length ? store.notes.filter(item => widget.selected_ids!.includes(item.id)) : store.notes.slice(0, 2) }
function chosenNote(widget: HomeWidget) { return widget.selected_ids?.length?store.notes.find(item=>item.id===widget.selected_ids![0]):store.notes[0] }
function chosenCountdowns(widget: HomeWidget) { return widget.selected_ids?.length?widget.selected_ids.map(id=>store.countdowns.find(item=>item.id===id)).filter((item):item is NonNullable<typeof item>=>!!item):upcoming.value }
function visibleCountdowns(widget: HomeWidget) { return chosenCountdowns(widget) }
const libraryTab=ref<Resource['kind']>('app')
const libraryItems=computed(()=>store.resources.filter(item=>item.kind===libraryTab.value))
function libraryTabKeydown(event:KeyboardEvent) { const kinds=['app','web','file'] as const; const index=kinds.indexOf(libraryTab.value); if(event.key!=='ArrowLeft'&&event.key!=='ArrowRight')return;event.preventDefault();libraryTab.value=kinds[(index+(event.key==='ArrowRight'?1:-1)+kinds.length)%kinds.length];const tabs=event.currentTarget as HTMLElement;void nextTick(()=>tabs.querySelector<HTMLElement>('[aria-selected="true"]')?.focus()) }
function launchItems(widget: HomeWidget) { return widget.selected_ids?.length ? widget.selected_ids.map(id=>store.resources.find(item=>item.id===id)).filter((item): item is Resource=>!!item) : store.resources.filter(item=>item.is_favorite) }
function toolItems(widget: HomeWidget) { return widget.selected_ids?.length ? builtinTools.filter(item=>widget.selected_ids!.includes(item.id)) : builtinTools }
async function launchTool(id:string) { try { await openTool(id) } catch(error) { store.notify(`无法打开工具：${String(error)}`) } }
function choices(widget:HomeWidget) { return widget.kind==='sticky'?store.stickies:widget.kind==='note'?store.notes:store.countdowns }
function selectContent(widget:HomeWidget,id:string) { const selected=widget.selected_ids??[]; widget.selected_ids=['countdowns','quick_launch','quick_tools'].includes(widget.kind)?(selected.includes(id)?selected.filter(x=>x!==id):[...selected,id]):['sticky','note'].includes(widget.kind)?(selected.includes(id)?selected.filter(x=>x!==id):selected.length<2?[...selected,id]:selected):[id] }
function selectLayoutContent(widget:HomeWidget,id:string) { const previous=[...(widget.selected_ids??[])]; selectContent(widget,id); if(previous.join('\0')!==widget.selected_ids?.join('\0')) { const next=widget.selected_ids; widget.selected_ids=previous; rememberLayout(); widget.selected_ids=next } }
function resetLayoutContent(widget:HomeWidget) { if(widget.selected_ids?.length){rememberLayout();widget.selected_ids=[]} }
const lastSticky = computed(() => dragWidget.value?chosenSticky(dragWidget.value):store.stickies[0])
const lastNote = computed(() => dragWidget.value?chosenNote(dragWidget.value):store.notes[0])
const recentResources = computed(() => store.resources
  .filter(item => item.last_opened_at)
  .sort((a, b) => Date.parse(b.last_opened_at!) - Date.parse(a.last_opened_at!))
)
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
const selectedModules = ref<HomeWidgetKind[]>([])
const dropTarget = ref('')
const newsDate = ref('')
const modulePreviews: Record<HomeWidgetKind,string> = { clock:'14:26:08 · 星期六', countdowns:'纪念日 · 还有 14 天', sticky:'随手记录想法和待办', note:'整理笔记，留住灵感', recent:'最近打开的应用和文件', weather:'22°C · 晴 · 体感 20°C', hitokoto:'一句话，遇见好心情。', news:'01 今日资讯\n02 每天一分钟，了解世界', poetry:'故人具鸡黍，邀我至田家。', library:'应用、网页和文件一览', quick_launch:'收藏的常用启动项', quick_tools:'日历、二维码与文本工具', calendar:'翻阅月历 · 快速回到今天', time:'14:26:08' }
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
    case 'time': return time.value
    case 'clock': return `${time.value} · ${longDate.value}`
    case 'countdowns': return (dragWidget.value?chosenCountdowns(dragWidget.value):upcoming.value).map(item => `${item.title} ${countdownStatus(item, now.value).text}`).join('\n') || '添加一个重要日期，让期待有个名字。'
    case 'sticky': return lastSticky.value?.content || '还没有便签，在这里记录待办或随手想法。'
    case 'note': return lastNote.value ? `${lastNote.value.title}\n${lastNote.value.content}` : '还没有笔记，开始记录和整理想法。'
    case 'recent': return recentResources.value.map(item => item.name).join(' · ') || '还没有打开记录。'
    default: return dragWidget.value ? modulePreviews[dragWidget.value.kind] : ''
  }
})
const widgets = ref<HomeWidget[]>(store.settings.home_layout.map(normalizeHomeWidget))
const moduleIcons = { clock: Clock3, countdowns: CalendarDays, sticky: StickyNote, note: NotebookPen, recent: History, weather: CloudSun, hitokoto: Quote, news: Newspaper, poetry: Flower2, library: Grid2X2, quick_launch: Rocket, quick_tools: Wrench, calendar: CalendarDays, time: Clock3 }
function isOnline(kind:HomeWidgetKind):kind is FeedKind{return ['weather','hitokoto','news','poetry'].includes(kind)}
const feedUpdates=ref<Record<string,string>>({})
const feedLoading=ref<Record<string,boolean>>({})
const refreshKeys=ref<Record<FeedKind,number>>({weather:0,hitokoto:0,news:0,poetry:0})
const modulePages: Partial<Record<HomeWidgetKind, PageId>> = { countdowns: 'countdowns', sticky: 'stickies', note: 'notes', recent: 'library', library: 'library', quick_launch: 'library' } as const
watch(() => store.settings.home_layout, value => { if (!editing.value) widgets.value = value.map(normalizeHomeWidget) }, { deep: true })
function beginEdit() { layoutUndo.value=[];widgets.value = store.settings.home_layout.map(normalizeHomeWidget); editing.value = true }
function openModules() { if (!editing.value) beginEdit(); selectedModules.value=[]; showModules.value = true }
function toggleModule(kind: HomeWidgetKind) { selectedModules.value=selectedModules.value.includes(kind)?selectedModules.value.filter(item=>item!==kind):[...selectedModules.value,kind] }
function addSelectedModules() { const kinds=selectedModules.value.filter(kind=>!widgets.value.some(widget=>widget.kind===kind));if(!kinds.length)return;rememberLayout();for(const kind of kinds)widgets.value.push(createModule(kind));showModules.value=false;selectedModules.value=[] }
function cancelEdit() { cancelDrag(); widgets.value = store.settings.home_layout.map(normalizeHomeWidget); editing.value = false; showModules.value = false }
function createModule(kind: HomeWidgetKind): HomeWidget { return { id: crypto.randomUUID(), kind, size: fixedWidgetSize(kind) ? '1x1' : ['news','calendar'].includes(kind) ? '2x2' : ['recent','library','quick_launch','quick_tools'].includes(kind)||isOnline(kind) ? '2x1' : '1x1' } }
function resize(widget: HomeWidget, size: HomeWidgetSize) { if(fixedWidgetSize(widget.kind)||widget.size===size)return;rememberLayout();widget.size = size }
function startDrag(event: PointerEvent, widget: HomeWidget) {
  if (!editing.value || saving.value || event.button !== 0) return
  const card = (event.currentTarget as HTMLElement).closest<HTMLElement>('[data-widget-id]')
  if (!card) return
  event.preventDefault()
  const rect = card.getBoundingClientRect()
  const capture = card.parentElement
  pendingDrag = { id: widget.id, pointerId: event.pointerId, startX: event.clientX, startY: event.clientY, original: widgets.value.map(normalizeHomeWidget), capture }
  capture?.setPointerCapture(event.pointerId)
  dragPosition.value = { x: event.clientX, y: event.clientY, offsetX: event.clientX - rect.left, offsetY: event.clientY - rect.top, width: rect.width, height: rect.height }
  lastPreviewAt = 0
}
function pointerMove(event: PointerEvent) {
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
  const dx = (event.clientX - (rect.left + rect.width / 2)) / rect.width
  const dy = (event.clientY - (rect.top + rect.height / 2)) / rect.height
  const delta = Math.abs(dy) > Math.abs(dx) ? dy : dx
  if (Math.abs(delta) < 0.08) return
  dropTarget.value=target.dataset.widgetId||''
  const next = previewWidgetMove(widgets.value, draggedId.value, target.dataset.widgetId!, delta > 0)
  if (next !== widgets.value) { widgets.value = next; lastPreviewAt = performance.now() }
}
function distance(element: HTMLElement, event: PointerEvent) { const rect=element.getBoundingClientRect(); return Math.hypot(event.clientX-Math.max(rect.left,Math.min(rect.right,event.clientX)),event.clientY-Math.max(rect.top,Math.min(rect.bottom,event.clientY))) }
function stopAutoScroll() { dropTarget.value=''; cancelAnimationFrame(autoScrollFrame); autoScrollFrame=0; lastPointerEvent=null }
function autoScroll() {
  if (!draggedId.value || !lastPointerEvent) return
  const container=document.querySelector<HTMLElement>('[data-home-scroll]')
  if (container) {
    const rect=container.getBoundingClientRect()
    const y=dragPosition.value.y
    const delta=y<rect.top+48?-10:y>rect.bottom-48?10:0
    if(delta){const before=container.scrollTop;container.scrollTop+=delta;if(container.scrollTop!==before)pointerMove(lastPointerEvent)}
  }
  autoScrollFrame=requestAnimationFrame(autoScroll)
}
function releasePointer() { if(pendingDrag?.capture?.hasPointerCapture(pendingDrag.pointerId))pendingDrag.capture.releasePointerCapture(pendingDrag.pointerId) }
function pointerUp(event: PointerEvent) { if (pendingDrag?.pointerId===event.pointerId) { if(draggedId.value && pendingDrag.original.some((item,index)=>widgets.value[index]?.id!==item.id))layoutUndo.value.push(pendingDrag.original);stopAutoScroll(); releasePointer(); pendingDrag=null; draggedId.value=null } }
function cancelDrag() { stopAutoScroll(); if (pendingDrag && draggedId.value) widgets.value=pendingDrag.original; releasePointer(); pendingDrag=null; draggedId.value=null }
function dragKeydown(event: KeyboardEvent) { if (event.key==='Escape' && pendingDrag) { event.preventDefault();cancelDrag() } }
function moveByKeyboard(event: KeyboardEvent, widget: HomeWidget) { const delta=['ArrowLeft','ArrowUp'].includes(event.key)?-1:['ArrowRight','ArrowDown'].includes(event.key)?1:0; if(delta&&!saving.value){event.preventDefault();const index=widgets.value.findIndex(item=>item.id===widget.id);const next=moveWidget(widgets.value,index,index+delta);if(next!==widgets.value){rememberLayout();widgets.value=next}} }
async function persistLayout() { saving.value = true; try { await store.setSetting('home_layout', widgets.value.map(normalizeHomeWidget)); editing.value = false; showModules.value = false; store.notify('首页布局已保存') } finally { saving.value = false } }
async function saveLayout() { try { await persistLayout() } catch(error) { store.notify(`保存首页布局失败：${String(error)}`) } }
</script>

<template>
  <div class="@container flex min-h-0 w-full flex-1 flex-col">
    <Dialog v-model:open="captureOpen"><DialogContent><DialogHeader><DialogTitle>快速记录</DialogTitle><DialogDescription>保存想法、笔记或重要日期。</DialogDescription></DialogHeader><QuickCapture @saved="captureOpen=false"/></DialogContent></Dialog>
    <Dialog :open="!!moduleSettings" @update:open="value=>{if(!value)moduleSettings=null}"><DialogContent><DialogHeader><DialogTitle>设置卡片内容</DialogTitle><DialogDescription>选择首页展示的记录，修改后保存生效。</DialogDescription></DialogHeader>
      <template v-if="moduleSettings&&(moduleSettings.kind==='sticky'||moduleSettings.kind==='note')">
        <div class="flex items-center justify-between gap-4 rounded-xl border border-border p-4"><div><p class="text-sm font-medium">自动展示最新两条</p><p class="mt-1 text-xs text-muted-foreground">根据{{ moduleSettings.kind==='sticky'?'便签':'笔记' }}的更新时间自动更新</p></div><Switch v-model="contentAuto" :aria-label="`自动展示最新两条${moduleSettings.kind==='sticky'?'便签':'笔记'}`" /></div>
        <div v-if="!contentAuto" class="space-y-2"><div class="flex items-center justify-between text-sm"><span class="font-medium">选择{{ moduleSettings.kind==='sticky'?'便签':'笔记' }}</span><span class="text-xs text-muted-foreground">已选 {{ moduleSettings.selected_ids?.length??0 }} / 2</span></div><div class="max-h-64 space-y-1 overflow-y-auto rounded-xl border border-border p-2"><label v-for="item in (moduleSettings.kind==='sticky'?store.stickies:store.notes)" :key="item.id" class="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 hover:bg-muted/60"><Checkbox :model-value="moduleSettings.selected_ids?.includes(item.id)??false" :disabled="!moduleSettings.selected_ids?.includes(item.id)&&(moduleSettings.selected_ids?.length??0)>=2" @update:model-value="selectContent(moduleSettings!,item.id)" /><span class="min-w-0 flex-1"><span class="block truncate text-sm font-medium">{{ item.title||'无标题' }}</span><span class="block truncate text-xs text-muted-foreground">{{ item.content?.replace(/\s+/g,' ')||'暂无内容' }}</span></span></label><p v-if="!choices(moduleSettings).length" class="p-3 text-sm text-muted-foreground">暂无{{ moduleSettings.kind==='sticky'?'便签':'笔记' }}，请先创建。</p></div><p class="text-xs text-muted-foreground">请选择两条内容。</p></div>
      </template>
      <template v-else-if="moduleSettings&&(moduleSettings.kind==='quick_launch'||moduleSettings.kind==='quick_tools')">
        <div class="flex items-center justify-between gap-4 rounded-xl border p-4"><div><p class="text-sm font-medium">{{ moduleSettings.kind==='quick_launch'?'自动显示收藏的启动项':'自动显示全部工具' }}</p><p class="mt-1 text-xs text-muted-foreground">关闭后可选择要显示的项目</p></div><Switch v-model="contentAuto" aria-label="自动显示内容" /></div>
        <div v-if="!contentAuto" class="space-y-2"><p class="text-sm font-medium">选择显示内容</p><div class="max-h-64 space-y-1 overflow-y-auto rounded-xl border p-2"><label v-for="item in (moduleSettings.kind==='quick_launch'?store.resources.map(resource=>({id:resource.id,title:resource.name,description:resource.target})):builtinTools)" :key="item.id" class="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 hover:bg-muted/60"><Checkbox :model-value="moduleSettings.selected_ids?.includes(item.id)??false" @update:model-value="selectContent(moduleSettings!,item.id)" /><span class="min-w-0 flex-1"><span class="block truncate text-sm font-medium">{{ item.title }}</span><span class="block truncate text-xs text-muted-foreground">{{ item.description }}</span></span></label><p v-if="moduleSettings.kind==='quick_launch'&&!store.resources.length" class="p-3 text-sm text-muted-foreground">应用库暂无启动项，请先添加。</p></div><p class="text-xs text-muted-foreground">已选择 {{ moduleSettings.selected_ids?.length??0 }} 项</p></div>
      </template>
      <FieldGroup v-else-if="moduleSettings"><Field><FieldLabel>显示内容</FieldLabel><Button variant="outline" @click="moduleSettings.selected_ids=[]">自动显示最近内容</Button><div class="max-h-64 overflow-y-auto"><Button v-for="item in choices(moduleSettings)" :key="item.id" variant="ghost" class="w-full justify-start" :aria-pressed="moduleSettings.selected_ids?.includes(item.id)" @click="selectContent(moduleSettings,item.id)">{{ moduleSettings.selected_ids?.includes(item.id)?'✓ ':'' }}{{ item.title||'无标题' }}</Button></div></Field><Field v-if="moduleSettings.kind==='countdowns'"><FieldLabel>自动显示数量</FieldLabel><Select v-model="countdownCount"><SelectTrigger><SelectValue/></SelectTrigger><SelectContent><SelectGroup><SelectItem v-for="count in [3,5,8]" :key="count" :value="String(count)">{{ count }} 个日期</SelectItem></SelectGroup></SelectContent></Select></Field></FieldGroup><Button :disabled="moduleSaving||((moduleSettings?.kind==='sticky'||moduleSettings?.kind==='note')&&!contentAuto&&(moduleSettings.selected_ids?.length??0)!==2)||((moduleSettings?.kind==='quick_launch'||moduleSettings?.kind==='quick_tools')&&!contentAuto&&!moduleSettings.selected_ids?.length)" @click="saveModule">{{ moduleSaving?'保存中…':'保存卡片设置' }}</Button></DialogContent></Dialog>
    <div data-home-scroll class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto overscroll-contain pb-4">
    <section v-if="!editing" class="flex flex-wrap items-center justify-end gap-4"><Button variant="outline" @click="captureOpen=true"><Plus data-icon="inline-start"/>快速记录</Button>
      <Button variant="outline" @click="beginEdit"><LayoutDashboard data-icon="inline-start" />编辑首页</Button>

    </section>
    <Dialog v-model:open="layoutDialog"><DialogContent class="max-h-[85dvh] overflow-y-auto sm:max-w-2xl"><DialogHeader><DialogTitle>选择布局模板</DialogTitle><DialogDescription>选择已保存的布局或预设模板，应用后可继续调整。</DialogDescription></DialogHeader>
      <div class="space-y-2"><p class="text-sm font-medium">保存当前布局</p><div class="flex gap-2"><Input v-model="templateName" maxlength="40" placeholder="输入布局名称" aria-label="布局名称" @keydown.enter.prevent="saveCurrentTemplate" /><Button variant="outline" :disabled="!templateName.trim()||templateBusy" @click="saveCurrentTemplate">保存</Button></div></div>
      <div class="space-y-2"><p class="text-sm font-medium">我的布局</p><div v-if="savedTemplates.length" class="max-h-48 space-y-1 overflow-y-auto"><div v-for="item in savedTemplates" :key="item.id" class="flex items-center gap-2 rounded-lg border p-2" :class="selectedTemplate===item.id?'border-primary bg-primary/5':''"><button type="button" class="min-w-0 flex-1 px-2 py-1 text-left" :aria-pressed="selectedTemplate===item.id" @click="selectedTemplate=item.id"><span class="block truncate text-sm font-medium">{{ item.name }}</span><span class="text-xs text-muted-foreground">{{ item.widgets.length }} 张卡片</span></button><ConfirmDialog title="删除布局模板？" :description="`将删除布局模板“${item.name}”，当前首页布局不受影响。`" confirm-text="删除" destructive @confirm="deleteTemplate(item.id)"><template #trigger><Button variant="ghost" size="icon-sm" :disabled="templateBusy" :aria-label="`删除布局模板 ${item.name}`"><Trash2 /></Button></template></ConfirmDialog></div></div><p v-else class="rounded-lg border border-dashed p-4 text-sm text-muted-foreground">还没有保存的布局，可先为当前布局命名并保存。</p></div>
      <div class="space-y-2"><p class="text-sm font-medium">预设模板</p><div class="grid gap-2 sm:grid-cols-3"><button v-for="item in presetTemplates" :key="item.id" type="button" :aria-pressed="selectedTemplate===`preset:${item.id}`" class="rounded-lg border p-3 text-left" :class="selectedTemplate===`preset:${item.id}`?'border-primary bg-primary/5':'hover:border-primary'" @click="selectedTemplate=`preset:${item.id}`"><span class="block text-sm font-medium">{{ item.name }}</span><span class="mt-1 block text-xs text-muted-foreground">{{ item.description }}</span></button></div></div>
      <div class="flex justify-end gap-2"><Button variant="ghost" @click="layoutDialog=false">取消</Button><Button :disabled="!selectedTemplate||templateBusy" @click="applyTemplate">应用</Button></div>
    </DialogContent></Dialog>
    <Dialog v-model:open="showModules"><DialogContent class="max-h-[85dvh] overflow-y-auto sm:max-w-3xl"><DialogHeader><DialogTitle>添加卡片</DialogTitle><DialogDescription>选择要添加的卡片，将按选择顺序添加到首页末尾。</DialogDescription></DialogHeader>
      <div class="grid grid-cols-2 gap-3 sm:grid-cols-4"><button v-for="(name,kind) in widgetNames" :key="kind" type="button" :disabled="widgets.some(item=>item.kind===kind)" :aria-pressed="selectedModules.includes(kind)" class="flex min-h-24 flex-col gap-2 rounded-xl border bg-background p-3 text-left transition-colors disabled:cursor-not-allowed disabled:opacity-50" :class="selectedModules.includes(kind)?'border-primary bg-primary/5':'hover:border-primary'" @click="toggleModule(kind)"><span class="flex items-center gap-2 text-sm font-medium"><component :is="moduleIcons[kind]" class="size-4 text-primary" />{{ name }}<span v-if="widgets.some(item=>item.kind===kind)" class="ml-auto text-xs text-muted-foreground">已添加</span><span v-else-if="selectedModules.includes(kind)" class="ml-auto text-xs text-primary">已选择</span></span><span class="whitespace-pre-line text-xs leading-5 text-muted-foreground">{{ modulePreviews[kind] }}</span></button></div>
      <div class="flex justify-end gap-2"><Button variant="ghost" @click="showModules=false">取消</Button><Button :disabled="!selectedModules.length||saving" @click="addSelectedModules">添加{{ selectedModules.length?`（${selectedModules.length}）`:'' }}</Button></div>
    </DialogContent></Dialog>
    <TransitionGroup v-if="widgets.length" tag="div" data-home-grid move-class="transition-transform duration-150 ease-out motion-reduce:transition-none" class="grid grid-cols-2 auto-rows-[calc(162px+1rem)] gap-[16px] @min-[64rem]:grid-cols-4">
      <Card v-for="widget in widgets" :key="widget.id" :data-widget-id="widget.id" class="@container relative min-h-0 min-w-0 gap-0 overflow-hidden py-0" :class="[widgetClasses[widget.size],draggedId===widget.id?'opacity-30 ring-2 ring-primary ring-offset-2 ring-offset-background':'']">
        <div v-if="editing&&dropTarget===widget.id" class="pointer-events-none absolute inset-x-0 top-0 z-10 border-t-4 border-primary"><span class="rounded-b bg-primary px-2 py-1 text-xs text-primary-foreground">移到这里</span></div>
        <div v-if="editing" class="flex h-[40px] shrink-0 items-center justify-between gap-1 border-b border-border bg-muted/50 px-3">
          <Button variant="ghost" size="icon-sm" class="touch-none cursor-grab active:cursor-grabbing" :disabled="saving" :aria-label="`拖动${widgetNames[widget.kind]}`" @pointerdown="startDrag($event,widget)" @keydown="moveByKeyboard($event,widget)"><GripVertical /></Button>
          <span class="truncate text-sm font-semibold">{{ widgetNames[widget.kind] }}</span>
          <div class="flex shrink-0 gap-1">
            <DropdownMenu v-if="['sticky','note','countdowns'].includes(widget.kind)"><DropdownMenuTrigger as-child><Button variant="ghost" size="icon-xs" aria-label="选择模块内容" title="选择显示内容"><ListFilter/></Button></DropdownMenuTrigger><DropdownMenuContent align="end" class="max-h-80 w-60 overflow-y-auto"><DropdownMenuLabel>选择显示的内容</DropdownMenuLabel><DropdownMenuGroup><DropdownMenuItem @select="resetLayoutContent(widget)">自动显示{{ widget.kind==='countdowns'?'最近日期':'最新内容' }}</DropdownMenuItem></DropdownMenuGroup><DropdownMenuGroup><DropdownMenuCheckboxItem v-for="item in choices(widget)" :key="item.id" :model-value="widget.selected_ids?.includes(item.id)??false" @select="$event.preventDefault()" @update:model-value="selectLayoutContent(widget,item.id)">{{ item.title||'无标题' }}</DropdownMenuCheckboxItem></DropdownMenuGroup><DropdownMenuLabel v-if="!choices(widget).length">暂无内容，请先创建</DropdownMenuLabel></DropdownMenuContent></DropdownMenu>
            <TooltipProvider v-if="fixedWidgetSize(widget.kind)" :delay-duration="200"><Tooltip><TooltipTrigger as-child><span tabindex="0" role="button" aria-disabled="true" aria-label="1 乘 1，布局暂不可调整" class="inline-flex h-7 items-center rounded-md px-2 text-xs tabular-nums text-muted-foreground">1×1</span></TooltipTrigger><TooltipContent>布局暂不可调整</TooltipContent></Tooltip></TooltipProvider>
            <DropdownMenu v-else>
              <DropdownMenuTrigger as-child><Button variant="ghost" size="sm" class="h-7 px-2 text-xs tabular-nums" :disabled="saving" :aria-label="`调整${widgetNames[widget.kind]}尺寸，当前${widget.size.replace('x','乘')}`" title="调整布局尺寸">{{ widget.size.replace('x','×') }}</Button></DropdownMenuTrigger>
              <DropdownMenuContent align="end"><DropdownMenuLabel>布局尺寸 · 宽 × 高</DropdownMenuLabel><DropdownMenuRadioGroup :model-value="widget.size" @update:model-value="value=>resize(widget,value as HomeWidgetSize)"><DropdownMenuRadioItem v-for="size in widgetSizes" :key="size" :value="size">{{ size.replace('x','×') }}</DropdownMenuRadioItem></DropdownMenuRadioGroup></DropdownMenuContent>
            </DropdownMenu>
            <Button variant="danger" size="icon-xs" :disabled="saving" :aria-label="`移除${widgetNames[widget.kind]}模块`" @click="removeWidget(widget.id)"><Trash2 /></Button>
          </div>
        </div>
        <div v-if="!editing" class="flex h-[40px] shrink-0 items-center justify-between gap-2 border-b border-border px-4">
          <div class="flex min-w-0 items-center gap-3"><span class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary"><component :is="moduleIcons[widget.kind]" class="size-5" /></span><h2 class="shrink-0 font-semibold">{{ widgetNames[widget.kind] }}</h2><span v-if="['weather','hitokoto','poetry'].includes(widget.kind)&&feedUpdates[widget.id]" class="truncate text-xs text-muted-foreground">{{ feedUpdates[widget.id] }}</span><span v-if="widget.kind==='news'&&newsDate" class="truncate text-sm text-muted-foreground">{{ newsDate }} · 每天一分钟，了解世界</span></div>
          <div class="ml-auto flex shrink-0 items-center gap-1">
          <DropdownMenu v-if="widget.kind==='library'&&widget.size==='1x1'"><DropdownMenuTrigger as-child><Button variant="ghost" size="icon-sm" :aria-label="`选择应用库类型，当前${kindLabels[libraryTab]}`" title="选择应用库类型"><ListFilter /></Button></DropdownMenuTrigger><DropdownMenuContent align="end"><DropdownMenuLabel>应用库类型</DropdownMenuLabel><DropdownMenuRadioGroup :model-value="libraryTab" @update:model-value="value=>libraryTab=value as Resource['kind']"><DropdownMenuRadioItem v-for="kind in (['app','web','file'] as const)" :key="kind" :value="kind">{{ kindLabels[kind] }}</DropdownMenuRadioItem></DropdownMenuRadioGroup></DropdownMenuContent></DropdownMenu>
          <div v-else-if="widget.kind==='library'" role="tablist" aria-label="应用库类型" class="flex items-center rounded-lg bg-muted/60 p-0.5" @keydown="libraryTabKeydown"><button v-for="kind in (['app','web','file'] as const)" :key="kind" type="button" role="tab" :aria-selected="libraryTab===kind" :tabindex="libraryTab===kind?0:-1" class="rounded-md px-2 py-1 text-xs text-muted-foreground hover:text-foreground" :class="libraryTab===kind?'bg-background font-medium text-foreground shadow-sm':''" @click="libraryTab=kind">{{ kindLabels[kind] }}</button></div>
          <Button v-if="!editing&&['sticky','note','countdowns','quick_launch','quick_tools'].includes(widget.kind)" variant="ghost" size="icon-sm" aria-label="设置卡片内容" @click="configure(widget)"><ListFilter/></Button><Button v-if="isOnline(widget.kind)" variant="ghost" size="icon-sm" :disabled="feedLoading[widget.id]" :aria-label="feedLoading[widget.id]?`正在刷新${widgetNames[widget.kind]}`:`刷新${widgetNames[widget.kind]}`" :title="feedLoading[widget.id]?`正在刷新${widgetNames[widget.kind]}`:`刷新${widgetNames[widget.kind]}`" @click="refreshKeys[widget.kind]++"><RefreshCw :class="feedLoading[widget.id]?'animate-spin':''" /></Button><Button v-else-if="modulePages[widget.kind]" variant="ghost" size="icon-sm" :aria-label="`打开${widgetNames[widget.kind]}`" @click="emit('navigate',modulePages[widget.kind]!)"><ArrowUpRight /></Button>
          </div>
        </div>
        <div class="min-h-0 flex-1 overflow-hidden p-2" :inert="editing"><div class="flex h-full min-h-0 flex-col gap-[4px] overflow-y-auto pr-1" :class="['countdowns','sticky','note','library','quick_launch','quick_tools','recent'].includes(widget.kind)?(widget.size.endsWith('x2')?'max-h-[306px]':'max-h-[120px]'):''">
          <HomeFeed v-if="isOnline(widget.kind)" :kind="widget.kind" :size="widget.size" :refresh-key="refreshKeys[widget.kind]" @updated="value=>feedUpdates[widget.id]=value" @metadata="value=>{if(widget.kind==='news')newsDate=value}" @loading="value=>feedLoading[widget.id]=value" /><template v-else-if="widget.kind==='clock'"><div class="flex min-h-0 flex-1 flex-col gap-2"><div class="flex w-full gap-1" :class="widget.size==='2x2'?'shrink-0 flex-row items-center justify-between gap-3':'flex-1 flex-col justify-center'"><p class="text-2xl font-semibold tabular-nums tracking-tight @min-[20rem]:text-4xl">{{ time }}</p><div v-if="widget.size==='2x2'" class="flex shrink-0 flex-col items-center text-center text-sm text-muted-foreground"><span>{{ now.getFullYear() }} 年 {{ now.getMonth()+1 }} 月 {{ now.getDate() }} 日</span><span>{{ now.toLocaleDateString('zh-CN',{weekday:'long'}) }}</span></div><p v-else class="text-sm text-muted-foreground">{{ longDate }}</p></div><div v-if="widget.size==='2x2'" class="flex min-h-0 flex-1 flex-col px-3 py-2" :aria-label="`${clockCalendarMonth}日历`"><div class="flex shrink-0 items-center justify-between gap-1"><Button variant="ghost" size="icon-sm" aria-label="上个月" @click="moveClockCalendar(-1)"><ChevronLeft /></Button><span class="text-sm font-semibold tabular-nums">{{ clockCalendarMonth }}</span><div class="flex items-center"><Button variant="ghost" size="sm" class="px-2 text-xs" :disabled="clockCalendarIsCurrent" @click="clockCalendarToday">今天</Button><Button variant="ghost" size="icon-sm" aria-label="下个月" @click="moveClockCalendar(1)"><ChevronRight /></Button></div></div><div class="grid min-h-0 flex-1 grid-cols-7 auto-rows-fr text-center text-xs"><span v-for="day in ['一','二','三','四','五','六','日']" :key="day" class="flex items-center justify-center text-muted-foreground">{{ day }}</span><span v-for="(day,index) in currentMonthDays" :key="index" class="flex min-h-7 items-center justify-center"><span v-if="day" class="flex size-7 items-center justify-center rounded-full tabular-nums" :class="clockCalendarIsCurrent&&day===now.getDate()?'bg-primary font-semibold text-primary-foreground':'transition-colors hover:bg-muted'">{{ day }}</span></span></div></div></div></template>
          <template v-else-if="widget.kind==='time'"><div class="flex min-h-0 flex-1 items-center justify-center"><p class="text-3xl font-semibold tabular-nums tracking-tight @min-[20rem]:text-5xl">{{ time }}</p></div></template>
          <template v-else-if="widget.kind==='calendar'"><div class="flex min-h-0 flex-1 flex-col"><div class="flex shrink-0 items-center justify-between gap-1"><Button variant="ghost" size="icon-sm" aria-label="上个月" @click="moveCalendar(-1)"><ChevronLeft /></Button><span class="text-sm font-semibold tabular-nums">{{ calendarMonth }}</span><div class="flex items-center"><Button variant="ghost" size="sm" class="px-2 text-xs" :disabled="calendarIsCurrent" @click="calendarToday">今天</Button><Button variant="ghost" size="icon-sm" aria-label="下个月" @click="moveCalendar(1)"><ChevronRight /></Button></div></div><div class="grid flex-1 grid-cols-7 auto-rows-fr text-center text-xs" :aria-label="`${calendarMonth}日历`"><span v-for="day in ['一','二','三','四','五','六','日']" :key="day" class="flex min-h-6 items-center justify-center text-muted-foreground">{{ day }}</span><span v-for="(day,index) in calendarDays" :key="index" class="flex min-h-6 items-center justify-center"><span v-if="day" class="flex size-6 items-center justify-center rounded-full tabular-nums" :class="calendarIsCurrent&&day===now.getDate()?'bg-primary font-semibold text-primary-foreground':'transition-colors hover:bg-muted'">{{ day }}</span></span></div></div></template>
          <template v-else-if="widget.kind==='countdowns'">
            <div v-if="visibleCountdowns(widget).length" class="grid gap-[4px]" :class="widget.size==='2x2'?'@min-[32rem]:grid-cols-2':''">
              <Button v-for="item in visibleCountdowns(widget)" :key="item.id" variant="ghost" class="h-[58px] min-w-0 items-center justify-between gap-3 rounded-lg bg-muted/40 px-3 py-2 text-left" @click="openContent('countdowns',item.id)"><span class="min-w-0"><span class="block truncate text-sm font-medium">{{ item.title }}</span><span class="mt-0.5 block text-xs text-muted-foreground">{{ format(countdownTarget(item,now),'yyyy-MM-dd') }}</span></span><span class="shrink-0 text-sm font-semibold text-primary">{{ countdownStatus(item,now).text }}</span></Button>
            </div><p v-else class="flex flex-1 items-center justify-center text-center text-sm leading-6 text-muted-foreground">{{ widget.selected_ids?.length?'选中的日期已删除，请重新选择。':'添加一个重要日期，让期待有个名字。' }}</p>
          </template>
          <template v-else-if="widget.kind==='sticky'">
            <Button v-for="item in visibleStickies(widget)" :key="item.id" variant="ghost" class="w-full shrink-0 flex-col items-start justify-center gap-0.5 rounded-lg px-3 py-1.5 text-left whitespace-normal" :class="widget.size.endsWith('x2')?'h-[120px]':'h-[58px]'" @click="openContent('stickies',item.id)">
              <span class="w-full truncate text-sm font-medium">{{ item.title||'无标题便签' }}</span>
              <span class="w-full text-xs leading-5 text-muted-foreground" :class="widget.size.endsWith('x2')?'line-clamp-4 whitespace-pre-wrap break-words':'truncate'">{{ (widget.size.endsWith('x2')?item.content:item.content?.replace(/\s+/g,' '))||'这条便签还没有内容。' }}</span>
            </Button>
            <p v-if="!visibleStickies(widget).length" class="flex flex-1 items-center justify-center text-center text-sm leading-6 text-muted-foreground">{{ widget.selected_ids?.length?'选中的便签已删除，请重新选择。':'留一块空白，给灵感落脚。' }}</p>
          </template>
          <template v-else-if="widget.kind==='note'">
            <Button v-for="item in visibleNotes(widget)" :key="item.id" variant="ghost" class="w-full shrink-0 flex-col items-start justify-center gap-0.5 rounded-lg px-3 py-1.5 text-left whitespace-normal" :class="widget.size.endsWith('x2')?'h-[120px]':'h-[58px]'" @click="openContent('notes',item.id)">
              <span class="w-full truncate text-sm font-medium">{{ item.title||'无标题笔记' }}</span>
              <span class="w-full text-xs leading-5 text-muted-foreground" :class="widget.size.endsWith('x2')?'line-clamp-4 whitespace-pre-wrap break-words':'truncate'">{{ (widget.size.endsWith('x2')?item.content:item.content?.replace(/\s+/g,' '))||'这篇笔记还没有内容。' }}</span>
            </Button>
            <p v-if="!visibleNotes(widget).length" class="flex flex-1 items-center justify-center text-center text-sm leading-6 text-muted-foreground">{{ widget.selected_ids?.length?'选中的笔记已删除，请重新选择。':'创建第一篇笔记，开始整理想法。' }}</p>
          </template>
          <template v-else-if="widget.kind==='library'">
            <div v-if="libraryItems.length" role="tabpanel" class="grid grid-cols-[repeat(auto-fill,minmax(4.5rem,1fr))] gap-[4px]"><Button v-for="item in libraryItems" :key="item.id" variant="ghost" class="h-[58px] min-w-0 flex-col justify-center gap-1 rounded-xl px-1 py-1" :disabled="openingId===item.id" :aria-label="`打开 ${item.name}`" :title="item.name" @click="launchRecent(item)"><img v-if="resourceImage(store.settings.client,item)" :src="resourceImage(store.settings.client,item)!" alt="" class="size-6 shrink-0 object-contain" /><component :is="resourceIcons[item.kind]" v-else class="size-6 shrink-0 text-primary" /><span class="w-full truncate text-center text-xs">{{ item.name }}</span></Button></div><p v-else role="tabpanel" class="flex flex-1 items-center justify-center text-center text-sm text-muted-foreground">暂无{{ kindLabels[libraryTab] }}，可在应用库中添加。</p>
          </template>
          <template v-else-if="widget.kind==='quick_launch'">
            <div v-if="launchItems(widget).length" class="grid grid-cols-[repeat(auto-fill,minmax(4.5rem,1fr))] gap-[4px]"><Button v-for="item in launchItems(widget)" :key="item.id" variant="ghost" class="h-[58px] min-w-0 flex-col justify-center gap-1 rounded-xl px-1 py-1" :disabled="openingId===item.id" :aria-label="`打开 ${item.name}`" :title="item.name" @click="launchRecent(item)"><img v-if="resourceImage(store.settings.client,item)" :src="resourceImage(store.settings.client,item)!" alt="" class="size-6 shrink-0 object-contain" /><component :is="resourceIcons[item.kind]" v-else class="size-6 shrink-0 text-primary" /><span class="w-full truncate text-center text-xs">{{ item.name }}</span></Button></div><p v-else class="flex flex-1 items-center justify-center text-center text-sm text-muted-foreground">{{ widget.selected_ids?.length?'所选启动项已删除，请重新选择。':'在应用库收藏启动项，或通过右上角选择。' }}</p>
          </template>
          <template v-else-if="widget.kind==='quick_tools'">
            <div v-if="toolItems(widget).length" class="grid grid-cols-[repeat(auto-fill,minmax(4.5rem,1fr))] gap-[4px]"><Button v-for="item in toolItems(widget)" :key="item.id" variant="ghost" class="h-[58px] min-w-0 flex-col justify-center gap-1 rounded-xl px-1 py-1" :aria-label="`打开 ${item.title}`" :title="item.title" @click="launchTool(item.id)"><component :is="item.icon" class="size-6 shrink-0" /><span class="w-full truncate text-center text-xs">{{ item.title }}</span></Button></div><p v-else class="flex flex-1 items-center justify-center text-center text-sm text-muted-foreground">所选工具不可用，请重新选择。</p>
          </template>
          <template v-else>
            <div v-if="recentResources.length" class="grid grid-cols-[repeat(auto-fill,minmax(4.5rem,1fr))] gap-[4px]">
              <Button v-for="item in recentResources" :key="item.id" variant="ghost" class="h-[58px] min-w-0 flex-col justify-center gap-1 rounded-xl px-1 py-1 hover:bg-muted/70" :disabled="openingId===item.id" :aria-label="`打开 ${item.name}，${kindLabels[item.kind]}，${openedTime(item.last_opened_at)}`" :title="`${item.target}\n${kindLabels[item.kind]} · ${openedTime(item.last_opened_at)}`" @click="launchRecent(item)"><img v-if="resourceImage(store.settings.client,item)" :src="resourceImage(store.settings.client,item)!" alt="" class="size-6 shrink-0 object-contain" /><component :is="resourceIcons[item.kind]" v-else class="size-6 shrink-0 text-primary" /><span class="w-full truncate text-center text-sm">{{ openingId===item.id?'正在打开…':item.name }}</span></Button>
            </div><p v-else class="flex flex-1 items-center justify-center text-center text-sm text-muted-foreground">还没有打开记录。从应用库打开项目后，会自动显示在这里。</p>
          </template>
        </div></div>
      </Card>
    </TransitionGroup>
    <Card v-else data-home-grid><CardHeader><CardTitle>首页还没有模块</CardTitle><CardDescription>添加日期时间、天气、新闻、诗词和记录模块，组合你的首页。</CardDescription></CardHeader><CardContent><Button :disabled="saving" @click="openModules"><Plus data-icon="inline-start" />添加卡片</Button></CardContent></Card>
    </div>
    <div v-if="editing" class="z-30 flex shrink-0 flex-wrap items-center justify-between gap-2 border-t border-border bg-background py-3">      <div class="flex flex-wrap items-center gap-2">
        <Button variant="outline" :disabled="saving" @click="openModules"><Plus data-icon="inline-start" />添加卡片</Button>
        <Button variant="outline" :disabled="saving" @click="openLayoutTemplates">选择布局模板</Button>
        <ConfirmDialog title="恢复默认首页布局？" description="只恢复模块位置和尺寸，便签、笔记和应用库数据会保留。" confirm-text="恢复默认" @confirm="changeLayout(defaultHomeLayout())"><template #trigger><Button variant="outline" :disabled="saving"><RotateCcw data-icon="inline-start" />恢复默认</Button></template></ConfirmDialog>
      </div><div class="ml-auto flex shrink-0 items-center gap-2"><Button variant="ghost" :disabled="saving" @click="cancelEdit">取消</Button><Button :disabled="saving" @click="saveLayout">{{ saving?'保存中…':'保存布局' }}</Button></div></div>
    <Teleport to="body">
      <Card v-if="dragWidget" :style="ghostStyle" class="pointer-events-none fixed z-50 select-none gap-4 overflow-hidden border-primary bg-card/95 p-4 shadow-xl">
        <div class="flex items-center gap-3"><component :is="moduleIcons[dragWidget.kind]" class="size-5 text-primary" /><h2 class="font-semibold">{{ widgetNames[dragWidget.kind] }}</h2><span class="ml-auto text-xs text-muted-foreground">{{ dragWidget.size.replace('x','×') }}</span></div>
        <p class="line-clamp-6 whitespace-pre-wrap text-sm leading-6 text-muted-foreground">{{ dragPreviewText }}</p>
      </Card>
    </Teleport>
  </div>
</template>
