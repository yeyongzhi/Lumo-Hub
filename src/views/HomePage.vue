<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { AppWindow, ArrowUpRight, CalendarDays, Clock3, FileText, Globe, GripVertical, History, LayoutDashboard, Maximize2, ListFilter, NotebookPen, Plus, RotateCcw, StickyNote, Trash2 } from '@lucide/vue'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuLabel, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuCheckboxItem, DropdownMenuGroup, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { countdownStatus, formatLongDate } from '@/lib/dates'
import { useWorkspaceStore } from '@/stores/workspace'
import { defaultHomeLayout, moveWidget, previewWidgetMove, widgetClasses, widgetNames, widgetSizes } from '@/lib/home-layout'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import type { HomeWidget, HomeWidgetKind, HomeWidgetSize, PageId, Resource } from '@/lib/types'

const emit = defineEmits<{ navigate: [page: PageId] }>()
const store = useWorkspaceStore()
const now = ref(new Date())
let timer: ReturnType<typeof setInterval>
let unregisterSave: (() => void) | undefined
onMounted(() => { window.addEventListener('pointermove', pointerMove, { passive: false }); window.addEventListener('pointerup', pointerUp); window.addEventListener('pointercancel', cancelDrag); window.addEventListener('blur', cancelDrag); window.addEventListener('keydown', dragKeydown); timer = setInterval(() => { now.value = new Date() }, 1000); unregisterSave = store.registerPendingSave(async () => { if (editing.value) await persistLayout() }) })
onUnmounted(() => { stopAutoScroll(); window.removeEventListener('pointermove', pointerMove); window.removeEventListener('pointerup', pointerUp); window.removeEventListener('pointercancel', cancelDrag); window.removeEventListener('blur', cancelDrag); window.removeEventListener('keydown', dragKeydown); clearInterval(timer); unregisterSave?.() })
const longDate = computed(() => formatLongDate(now.value))
const time = computed(() => now.value.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' }))
const upcoming = computed(() => [...store.countdowns].sort((a, b) => countdownStatus(a, now.value).days - countdownStatus(b, now.value).days).slice(0, 3))
function chosenSticky(widget: HomeWidget) { return widget.selected_ids?.length?store.stickies.find(item=>item.id===widget.selected_ids![0]):store.stickies[0] }
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
    case 'sticky': return lastSticky.value?.content || '留一块空白，给灵感落脚。'
    case 'note': return lastNote.value ? `${lastNote.value.title}\n${lastNote.value.content}` : '创建第一篇笔记，开始整理想法。'
    case 'recent': return recentResources.value.map(item => item.name).join(' · ') || '还没有打开记录。'
    default: return ''
  }
})
const widgets = ref<HomeWidget[]>(store.settings.home_layout.map(item => ({ ...item, selected_ids: [...(item.selected_ids??[])] })))
const moduleIcons = { clock: Clock3, countdowns: CalendarDays, sticky: StickyNote, note: NotebookPen, recent: History }
const modulePages = { countdowns: 'countdowns', sticky: 'stickies', note: 'notes', recent: 'library' } as const
watch(() => store.settings.home_layout, value => { if (!editing.value) widgets.value = value.map(item => ({ ...item, selected_ids: [...(item.selected_ids??[])] })) }, { deep: true })
function beginEdit() { widgets.value = store.settings.home_layout.map(item => ({ ...item, selected_ids: [...(item.selected_ids??[])] })); editing.value = true }
function openModules() { if (!editing.value) beginEdit(); showModules.value = true }
function cancelEdit() { stopAutoScroll(); widgets.value = store.settings.home_layout.map(item => ({ ...item, selected_ids: [...(item.selected_ids??[])] })); editing.value = false; showModules.value = false; draggedId.value = null; pendingDrag = null }
function addModule(kind: HomeWidgetKind) { if (widgets.value.some(item => item.kind === kind)) return; widgets.value.push({ id: crypto.randomUUID(), kind, size: kind === 'recent' ? '2x1' : '1x1' }) }
function resize(widget: HomeWidget, size: HomeWidgetSize) { widget.size = size }
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
  const next = previewWidgetMove(widgets.value, draggedId.value, target.dataset.widgetId!, delta > 0)
  if (next !== widgets.value) { widgets.value = next; lastPreviewAt = performance.now() }
}
function distance(element: HTMLElement, event: PointerEvent) { const rect=element.getBoundingClientRect(); return Math.hypot(event.clientX-Math.max(rect.left,Math.min(rect.right,event.clientX)),event.clientY-Math.max(rect.top,Math.min(rect.bottom,event.clientY))) }
function stopAutoScroll() { cancelAnimationFrame(autoScrollFrame); autoScrollFrame=0; lastPointerEvent=null }
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
function pointerUp(event: PointerEvent) { if (pendingDrag?.pointerId===event.pointerId) { stopAutoScroll(); releasePointer(); pendingDrag=null; draggedId.value=null } }
function cancelDrag() { stopAutoScroll(); if (pendingDrag && draggedId.value) widgets.value=pendingDrag.original; releasePointer(); pendingDrag=null; draggedId.value=null }
function dragKeydown(event: KeyboardEvent) { if (event.key==='Escape' && pendingDrag) { event.preventDefault();cancelDrag() } }
function moveByKeyboard(event: KeyboardEvent, widget: HomeWidget) { const delta=['ArrowLeft','ArrowUp'].includes(event.key)?-1:['ArrowRight','ArrowDown'].includes(event.key)?1:0; if(delta&&!saving.value){event.preventDefault();const index=widgets.value.findIndex(item=>item.id===widget.id);widgets.value=moveWidget(widgets.value,index,index+delta)} }
async function persistLayout() { saving.value = true; try { await store.setSetting('home_layout', widgets.value.map(item => ({ ...item, selected_ids: [...(item.selected_ids??[])] }))); editing.value = false; showModules.value = false; store.notify('首页布局已保存') } finally { saving.value = false } }
async function saveLayout() { try { await persistLayout() } catch {} }
</script>

<template>
  <div class="@container flex w-full flex-col gap-6 pb-10">
    <section class="flex flex-wrap items-center justify-end gap-4">
      <Button v-if="!editing" variant="outline" @click="beginEdit"><LayoutDashboard data-icon="inline-start" />编辑首页</Button>
      <div v-else class="flex flex-wrap gap-2">
        <Button variant="outline" :disabled="saving" @click="showModules=!showModules"><Plus data-icon="inline-start" />添加模块</Button>
        <ConfirmDialog title="恢复默认首页布局？" description="只恢复模块位置和尺寸，便签、笔记和应用库数据会保留。" confirm-text="恢复默认" @confirm="widgets=defaultHomeLayout()"><template #trigger><Button variant="ghost" :disabled="saving"><RotateCcw data-icon="inline-start" />恢复默认</Button></template></ConfirmDialog>
        <Button variant="ghost" :disabled="saving" @click="cancelEdit">取消</Button><Button :disabled="saving" @click="saveLayout">{{ saving?'保存中…':'保存布局' }}</Button>
      </div>
    </section>
    <Card v-if="editing&&showModules">
      <CardHeader><CardTitle>添加首页模块</CardTitle><CardDescription>每种模块可添加一个，移除模块会保留原有数据。</CardDescription></CardHeader>
      <CardContent class="flex flex-wrap gap-3"><Button v-for="(name,kind) in widgetNames" :key="kind" variant="outline" :disabled="saving||widgets.some(item=>item.kind===kind)" @click="addModule(kind)"><component :is="moduleIcons[kind]" data-icon="inline-start" />{{ name }}{{ widgets.some(item=>item.kind===kind)?' · 已添加':'' }}</Button></CardContent>
    </Card>
    <p v-if="editing" class="text-sm text-muted-foreground">拖动左上角手柄实时调整位置，点击右上角尺寸图标选择宽 × 高，内容图标可指定便签、笔记或倒数日。窄窗口会自动调整为两列。</p>
    <TransitionGroup v-if="widgets.length" tag="div" data-home-grid move-class="transition-transform duration-150 ease-out motion-reduce:transition-none" class="grid grid-cols-2 auto-rows-[16rem] gap-4 @min-[64rem]:grid-cols-4">
      <Card v-for="widget in widgets" :key="widget.id" :data-widget-id="widget.id" class="min-h-0 min-w-0 gap-0 overflow-hidden py-0" :class="[widgetClasses[widget.size],draggedId===widget.id?'opacity-30 ring-2 ring-primary ring-offset-2 ring-offset-background':'']">
        <div v-if="editing" class="flex shrink-0 items-center justify-between gap-1 border-b border-border bg-muted/50 px-3 py-2">
          <Button variant="ghost" size="icon-sm" class="touch-none cursor-grab active:cursor-grabbing" :disabled="saving" :aria-label="`拖动${widgetNames[widget.kind]}`" @pointerdown="startDrag($event,widget)" @keydown="moveByKeyboard($event,widget)"><GripVertical /></Button>
          <span class="truncate text-xs text-muted-foreground">{{ widgetNames[widget.kind] }}</span>
          <div class="flex shrink-0 gap-1">
            <DropdownMenu v-if="['sticky','note','countdowns'].includes(widget.kind)"><DropdownMenuTrigger as-child><Button variant="ghost" size="icon-xs" aria-label="选择模块内容" title="选择显示内容"><ListFilter/></Button></DropdownMenuTrigger><DropdownMenuContent align="end" class="max-h-80 w-60 overflow-y-auto"><DropdownMenuLabel>选择显示的内容</DropdownMenuLabel><DropdownMenuGroup><DropdownMenuItem @select="widget.selected_ids=[]">自动显示{{ widget.kind==='countdowns'?'最近日期':'最新内容' }}</DropdownMenuItem></DropdownMenuGroup><DropdownMenuGroup><DropdownMenuCheckboxItem v-for="item in choices(widget)" :key="item.id" :model-value="widget.selected_ids?.includes(item.id)??false" @select="$event.preventDefault()" @update:model-value="selectContent(widget,item.id)">{{ item.title||'无标题' }}</DropdownMenuCheckboxItem></DropdownMenuGroup><DropdownMenuLabel v-if="!choices(widget).length">暂无内容，请先创建</DropdownMenuLabel></DropdownMenuContent></DropdownMenu>
            <DropdownMenu>
              <DropdownMenuTrigger as-child><Button variant="ghost" size="icon-xs" :disabled="saving" :aria-label="`调整${widgetNames[widget.kind]}尺寸，当前${widget.size.replace('x','乘')}`" title="调整布局尺寸"><Maximize2 /></Button></DropdownMenuTrigger>
              <DropdownMenuContent align="end"><DropdownMenuLabel>布局尺寸 · 宽 × 高</DropdownMenuLabel><DropdownMenuRadioGroup :model-value="widget.size" @update:model-value="value=>resize(widget,value as HomeWidgetSize)"><DropdownMenuRadioItem v-for="size in widgetSizes" :key="size" :value="size">{{ size.replace('x','×') }}</DropdownMenuRadioItem></DropdownMenuRadioGroup></DropdownMenuContent>
            </DropdownMenu>
            <Button variant="ghost" size="icon-xs" :disabled="saving" :aria-label="`移除${widgetNames[widget.kind]}模块`" @click="widgets=widgets.filter(item=>item.id!==widget.id)"><Trash2 /></Button>
          </div>
        </div>
        <div class="flex shrink-0 items-center justify-between gap-2 p-4">
          <div class="flex min-w-0 items-center gap-3"><span class="flex size-9 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary"><component :is="moduleIcons[widget.kind]" class="size-5" /></span><h2 class="truncate font-semibold">{{ widgetNames[widget.kind] }}</h2></div>
          <Button v-if="widget.kind!=='clock'" variant="ghost" size="icon-sm" :aria-label="`打开${widgetNames[widget.kind]}`" @click="emit('navigate',modulePages[widget.kind])"><ArrowUpRight /></Button>
        </div>
        <div class="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto px-4 pb-4">
          <template v-if="widget.kind==='clock'"><p class="mt-auto text-3xl font-semibold tabular-nums tracking-tight">{{ time }}</p><p class="text-sm text-muted-foreground">{{ longDate }}</p></template>
          <template v-else-if="widget.kind==='countdowns'"><div v-for="item in chosenCountdowns(widget)" :key="item.id" class="flex flex-wrap items-center justify-between gap-2"><span class="truncate text-sm">{{ item.title }}</span><span class="text-sm font-medium text-primary">{{ countdownStatus(item,now).text }}</span></div><p v-if="!chosenCountdowns(widget).length" class="my-auto text-sm text-muted-foreground">{{ widget.selected_ids?.length?'选中的日期已删除，请重新选择。':'添加一个重要日期，让期待有个名字。' }}</p></template>
          <template v-else-if="widget.kind==='sticky'"><p class="whitespace-pre-wrap text-sm leading-6 text-muted-foreground">{{ chosenSticky(widget)?.content||(widget.selected_ids?.length?'选中的便签已删除，请重新选择。':'留一块空白，给灵感落脚。') }}</p></template>
          <template v-else-if="widget.kind==='note'"><p v-if="chosenNote(widget)" class="truncate font-medium">{{ chosenNote(widget)?.title||'无标题笔记' }}</p><p class="whitespace-pre-wrap text-sm leading-6 text-muted-foreground">{{ chosenNote(widget)?chosenNote(widget)?.content||'这篇笔记还没有内容。':widget.selected_ids?.length?'选中的笔记已删除，请重新选择。':'创建第一篇笔记，开始整理想法。' }}</p></template>
          <template v-else>
            <div v-if="recentResources.length" class="grid grid-cols-[repeat(auto-fit,minmax(min(100%,11rem),1fr))] gap-2">
              <Button v-for="item in recentResources" :key="item.id" variant="outline" class="h-auto min-w-0 justify-start gap-2 p-3" :disabled="openingId!==null" :aria-label="`打开 ${item.name}`" :title="item.target" @click="launchRecent(item)"><img v-if="item.icon_data" :src="item.icon_data" alt="" class="size-7 shrink-0 object-contain" /><component :is="resourceIcons[item.kind]" v-else data-icon="inline-start" /><span class="flex min-w-0 flex-1 flex-col gap-1 text-left"><span class="truncate">{{ openingId===item.id?'正在打开…':item.name }}</span><span class="text-xs text-muted-foreground">{{ kindLabels[item.kind] }} · {{ openedTime(item.last_opened_at) }}</span></span></Button>
            </div><p v-else class="my-auto text-sm text-muted-foreground">还没有打开记录。从应用库打开项目后，会自动显示在这里。</p>
          </template>
        </div>
      </Card>
    </TransitionGroup>
    <Card v-else><CardHeader><CardTitle>首页还没有模块</CardTitle><CardDescription>添加日期时间、倒数日、便签、笔记或最近打开模块，组合你的首页。</CardDescription></CardHeader><CardContent><Button :disabled="saving" @click="openModules"><Plus data-icon="inline-start" />添加模块</Button></CardContent></Card>
    <Teleport to="body">
      <Card v-if="dragWidget" :style="ghostStyle" class="pointer-events-none fixed z-50 select-none gap-4 overflow-hidden border-primary bg-card/95 p-4 shadow-xl">
        <div class="flex items-center gap-3"><component :is="moduleIcons[dragWidget.kind]" class="size-5 text-primary" /><h2 class="font-semibold">{{ widgetNames[dragWidget.kind] }}</h2><span class="ml-auto text-xs text-muted-foreground">{{ dragWidget.size.replace('x','×') }}</span></div>
        <p class="line-clamp-6 whitespace-pre-wrap text-sm leading-6 text-muted-foreground">{{ dragPreviewText }}</p>
      </Card>
    </Teleport>
  </div>
</template>
