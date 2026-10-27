<script setup lang="ts">
import { openTool } from '@/lib/tools'
import QuickCapture from '@/components/QuickCapture.vue'
import SearchHighlight from '@/components/SearchHighlight.vue'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '@/components/ui/dialog'
import { confirmation, finishConfirmation, dismissConfirmation, askConfirmation } from '@/lib/confirm'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { exit } from '@tauri-apps/plugin-process'
import { open } from '@tauri-apps/plugin-dialog'
import { AppWindow, BookOpenText, CalendarDays, ChevronRight, CircleHelp, House, PanelLeftClose, PanelLeftOpen, Search, Settings, StickyNote, Plus, Home, CalendarPlus, StickyNote as StickyNoteIcon, NotebookPen, RefreshCw } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import WindowControls from '@/components/WindowControls.vue'
import { Input } from '@/components/ui/input'
import CountdownsPage from '@/views/CountdownsPage.vue'
import HomePage from '@/views/HomePage.vue'
import LibraryPage from '@/views/LibraryPage.vue'
import NotesPage from '@/views/NotesPage.vue'
import SettingsPage from '@/views/SettingsPage.vue'
import StickiesPage from '@/views/StickiesPage.vue'
import { readLocal, writeLocal } from '@/lib/local-files'
import { api } from '@/lib/api'
import { watchSystemTheme } from '@/lib/theme'
import type { PageId, SearchResult } from '@/lib/types'
import { useWorkspaceStore } from '@/stores/workspace'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'

const store=useWorkspaceStore()
const page=ref<PageId>('home')
const settingsRequest=ref<{checkUpdate:boolean;id:number}|null>(null)
async function openAbout(checkUpdate=false){
  if(!await navigate('settings'))return
  settingsRequest.value={checkUpdate,id:(settingsRequest.value?.id??0)+1}
}
const captureOpen=ref(false), shortcutsOpen=ref(false), captureText=ref('')
function showCapture(text=''){captureText.value=text;captureOpen.value=true}
async function addFromSearch(){if(await navigate('library'))await nextTick(()=>window.dispatchEvent(new Event('lumo-new')))}
async function newItem(){if(page.value==='library'||page.value==='countdowns'){window.dispatchEvent(new Event('lumo-new'));return}showCapture()}
function keyboard(event:KeyboardEvent){if(event.isComposing||event.repeat)return;if(event.key==='F1'){event.preventDefault();shortcutsOpen.value=true;return}if(event.ctrlKey&&event.key.toLowerCase()==='k'){event.preventDefault();document.querySelector<HTMLInputElement>('[aria-label="全局搜索"]')?.focus();return}if(event.ctrlKey&&event.key.toLowerCase()==='n'){event.preventDefault();void newItem();return}if(event.ctrlKey&&event.key.toLowerCase()==='s'){event.preventDefault();void store.flushPendingSaves().then(()=>store.notify('已保存')).catch(e=>store.notify(String(e)))}}
const search=ref('')
const searchResults=ref<SearchResult[]>([])
const searchOpen=ref(false)
function reloadPage(){window.location.reload()}
const sidebarDragWidth=ref<number|null>(null)
const sidebarWidth=computed(()=>sidebarDragWidth.value ?? store.settings.sidebar_width ?? 240)
const narrowSidebar=computed(()=>sidebarWidth.value<140)
let sidebarDragStartX=0, sidebarDragStartWidth=0
function startSidebarResize(event:PointerEvent){
  if(event.button!==0)return
  event.preventDefault()
  sidebarDragStartX=event.clientX
  sidebarDragStartWidth=sidebarWidth.value
  sidebarDragWidth.value=sidebarDragStartWidth
  ;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
}
function resizeSidebar(event:PointerEvent){
  if(sidebarDragWidth.value===null)return
  sidebarDragWidth.value=Math.round(Math.max(100,Math.min(280,sidebarDragStartWidth+event.clientX-sidebarDragStartX)))
}
function finishSidebarResize(){
  if(sidebarDragWidth.value===null)return
  const width=sidebarDragWidth.value
  void store.setSetting('sidebar_width',width).catch(()=>{})
  sidebarDragWidth.value=null
}
function resizeSidebarWithKeyboard(event:KeyboardEvent){
  if(event.key!=='ArrowLeft'&&event.key!=='ArrowRight')return
  event.preventDefault()
  void store.setSetting('sidebar_width',Math.max(100,Math.min(280,sidebarWidth.value+(event.key==='ArrowRight'?10:-10)))).catch(()=>{})
}
const sidebarCollapsed=ref(false)
let sidebarReady=false
readLocal<boolean>('lumo-sidebar-collapsed').then(value=>{sidebarCollapsed.value=value===true;sidebarReady=true}).catch(e=>store.notify(String(e)))
watch(sidebarCollapsed,value=>{if(sidebarReady)void writeLocal('lumo-sidebar-collapsed',value).catch(e=>store.notify(String(e)))})
const nav=[
  {id:'home',label:'首页',icon:House},
  {id:'countdowns',label:'倒数日',icon:CalendarDays},
  {id:'stickies',label:'便签',icon:StickyNote},
  {id:'notes',label:'笔记',icon:BookOpenText},
  {id:'library',label:'应用库',icon:AppWindow},
] as const
const pageComponents={home:HomePage,countdowns:CountdownsPage,stickies:StickiesPage,notes:NotesPage,library:LibraryPage,settings:SettingsPage}
const currentView=computed(()=>pageComponents[page.value])
let searchTimer:ReturnType<typeof setTimeout>|undefined
let removeThemeListener:(()=>void)|undefined
let removeCloseListener:(()=>void)|undefined
const extraListeners: (()=>void)[]=[]
let removeNavigationListener:(()=>void)|undefined
let removeResourceListener:(()=>void)|undefined
let removeCleanUninstallListener:(()=>void)|undefined
const activeSearchIndex=ref(0)

watch(search, value=>{
  if(searchTimer)clearTimeout(searchTimer)
  const query=value.trim()
  activeSearchIndex.value=0
  if(!query){searchResults.value=[];searchOpen.value=false;return}
  searchResults.value=[]
  searchTimer=setTimeout(async()=>{try{const results=await api.search(query);if(search.value.trim()===query){searchResults.value=results.sort((a,b)=>['resource','note','sticky','countdown'].indexOf(a.kind)-['resource','note','sticky','countdown'].indexOf(b.kind));searchOpen.value=true}}catch(e){store.notify(`搜索失败：${String(e)}`)}},250)
})

async function navigate(value:PageId){try{await store.flushPendingSaves()}catch{store.notify('内容尚未保存，请检查保存状态后重试');return false}page.value=value;searchOpen.value=false;return true}
async function openResult(result:SearchResult){if(result.id.startsWith("tool:")){try{await openTool(result.id.slice(5));searchOpen.value=false;search.value=""}catch(e){store.notify(String(e))}return}try{await store.flushPendingSaves()}catch{store.notify('内容尚未保存，请检查后重试');return}store.searchTarget={kind:result.kind,id:result.id};page.value=result.kind==='note'?'notes':result.kind==='sticky'?'stickies':result.kind==='countdown'?'countdowns':'library';searchOpen.value=false;search.value=''}
function handleSearchKeydown(event:KeyboardEvent){
  if(event.key==='Escape'){searchOpen.value=false;return}
  if(!searchResults.value.length)return
  if(event.key==='ArrowDown'){event.preventDefault();searchOpen.value=true;activeSearchIndex.value=(activeSearchIndex.value+1)%searchResults.value.length}
  else if(event.key==='ArrowUp'){event.preventDefault();searchOpen.value=true;activeSearchIndex.value=(activeSearchIndex.value+searchResults.value.length-1)%searchResults.value.length}
  else if(event.key==='Enter'&&searchOpen.value){event.preventDefault();void openResult(searchResults.value[activeSearchIndex.value])}
}
async function closeWindow(){try{await store.flushPendingSaves();await api.closeMain()}catch(e){store.notify(`无法关闭程序：${String(e)}`)}}
async function completeWelcomeAndShowLauncher(){await store.completeWelcome();await api.showLauncher()}
function startWindowDrag(event: MouseEvent){
  if(event.button!==0 || (event.target as HTMLElement).closest('button,input,textarea,select,a,[data-no-drag]'))return
  void getCurrentWindow().startDragging().catch(()=>{})
}
async function runCleanUninstall(){try{await api.cleanUninstall()}catch(e){store.notify(`卸载清理失败：${String(e)}`)}}
async function chooseRecoveryDirectory(){
  const path=await open({directory:true,multiple:false})
  if(typeof path!=='string')return
  try{await store.recoverDataDirectory(path);store.notify('数据目录已恢复')}
  catch(error){store.notify(`恢复失败：${String(error)}`)}
}
async function useDefaultRecoveryDirectory(){
  if(!await askConfirmation(`将数据路径设为默认目录：\n${store.bootstrap.default_data_dir}\n\n若该目录为空，会创建新的 Lumo-Hub 数据库。继续吗？`))return
  try{await store.recoverDataDirectory(store.bootstrap.default_data_dir);store.notify('已恢复默认数据目录')}
  catch(error){store.notify(`恢复失败：${String(error)}`)}
}
onMounted(async()=>{
  window.addEventListener('keydown',keyboard)
  const appWindow=getCurrentWindow()
  removeCloseListener=await appWindow.onCloseRequested(async event=>{
    event.preventDefault()
    try{await store.flushPendingSaves();await api.closeMain()}
    catch{store.notify('还有内容未保存，请检查保存状态后再次关闭。')}
  })
  removeCleanUninstallListener=await listen('lumo-clean-uninstall',async()=>{
    try{await store.flushPendingSaves();store.cleanUninstall=true}
    catch{store.notify('当前内容尚未保存，无法继续卸载清理。')}
  })
  removeResourceListener=await listen('lumo-resources-changed',async()=>{try{store.resources=await api.listResources()}catch{}})
  removeNavigationListener=await listen<{page:PageId;id:string|null}>('lumo-navigate',async({payload})=>{
    if(!await navigate(payload.page))return
    if(payload.id)store.searchTarget={kind:payload.page==='notes'?'note':payload.page==='stickies'?'sticky':payload.page==='countdowns'?'countdown':'resource',id:payload.id}
  })
  extraListeners.push(await listen('lumo-quit-requested',async()=>{try{await store.flushPendingSaves();await exit(0)}catch{store.notify('内容未保存，请打开窗口检查保存状态。')}}))
  extraListeners.push(await listen<string>('lumo-reminder',({payload})=>store.notify(payload)))
  extraListeners.push(await listen<string>('lumo-reminder-error',({payload})=>store.notify(`日期提醒发送失败：${payload}。可检查 Windows 通知设置。`)))
  await store.initialize()
  if(!store.startupError&&!store.recoveryError&&!store.cleanUninstall) { try{await api.openStartupView()}catch(e){store.notify(String(e))} }
  removeThemeListener=watchSystemTheme(()=>store.settings)
})
onUnmounted(()=>{window.removeEventListener('keydown',keyboard);extraListeners.forEach(unlisten=>unlisten());if(searchTimer)clearTimeout(searchTimer);removeThemeListener?.();removeCloseListener?.();removeCleanUninstallListener?.();removeResourceListener?.();removeNavigationListener?.()})
</script>

<template>
  <Dialog v-model:open="captureOpen"><DialogContent><DialogHeader><DialogTitle>快速记录</DialogTitle><DialogDescription>保存便签、笔记或重要日期。</DialogDescription></DialogHeader><QuickCapture :initial-text="captureText" @saved="item=>{captureOpen=false;openResult(item)}"/></DialogContent></Dialog>
  <Dialog v-model:open="shortcutsOpen"><DialogContent><DialogHeader><DialogTitle>快捷键与快速启动</DialogTitle><DialogDescription>从键盘快速找到内容和记录想法。</DialogDescription></DialogHeader><div class="flex flex-col gap-3 text-sm"><p>Ctrl + K：全局搜索</p><p>Ctrl + N：当前页面新建 / 快速记录</p><p>Ctrl + S：保存当前内容</p><p>Ctrl + Enter：保存快速记录</p><p>{{ store.settings.launcher.shortcut }}：唤出快捷面板</p><p>方向键选择，Enter 打开，Esc 收起快捷面板</p></div></DialogContent></Dialog>
  <ConfirmDialog :open="confirmation.open" title="确认操作" :description="confirmation.description" @update:open="value=>{if(!value)dismissConfirmation()}" @confirm="finishConfirmation(true)" />
  <div v-if="!store.initialized" class="relative flex min-h-screen items-center justify-center bg-background"><div class="absolute inset-x-0 top-0 h-12" @mousedown="startWindowDrag"></div><WindowControls class="absolute right-4 top-3" /><div class="flex items-center gap-3 text-sm text-muted-foreground"><span class="size-2 animate-pulse rounded-full bg-primary"></span>正在打开 Lumo-Hub…</div></div>
  <div v-else-if="store.startupError" class="relative flex min-h-screen items-center justify-center bg-background p-8">
    <div class="absolute inset-x-0 top-0 h-12" @mousedown="startWindowDrag"></div>
    <WindowControls class="absolute right-4 top-3" />
    <Card class="max-w-xl p-8"><h1 class="text-xl font-semibold">无法读取本地数据</h1><p class="mt-3 text-sm leading-6 text-muted-foreground">{{ store.startupError }}</p><p class="mt-4 text-xs text-muted-foreground">数据路径索引位于 `%LOCALAPPDATA%\Lumo-Hub\bootstrap.json`。请保留现有数据目录后再修复路径配置。</p></Card>
  </div>
  <main v-else-if="store.recoveryError" class="relative flex min-h-screen items-center justify-center bg-background p-8">
    <div class="absolute inset-x-0 top-0 h-12" @mousedown="startWindowDrag"></div>
    <WindowControls class="absolute right-4 top-3" />
    <Card class="w-full max-w-2xl p-8">
      <h1 class="text-2xl font-semibold">需要恢复数据目录</h1>
      <p class="mt-3 text-sm leading-6 text-muted-foreground">请检查下方错误。正式安装版的数据与程序在同一目录；选择旧数据目录可将其导入安装目录，现有数据不会自动删除。</p>
      <p class="mt-4 break-words rounded-xl bg-muted p-4 text-sm text-muted-foreground">{{ store.recoveryError }}</p>
      <p class="mt-4 break-all text-xs text-muted-foreground">数据位置：{{ store.bootstrap.default_data_dir }}</p>
      <div class="mt-8 flex flex-wrap gap-3"><Button @click="chooseRecoveryDirectory">导入已有数据目录</Button><Button variant="outline" @click="useDefaultRecoveryDirectory">修复后重试</Button></div>
      <p v-if="store.message" role="status" class="mt-4 text-sm text-destructive">{{ store.message }}</p>
    </Card>
  </main>
  <main v-else-if="store.cleanUninstall" class="relative flex min-h-screen items-center justify-center bg-background p-8">
    <div class="absolute inset-x-0 top-0 h-12" @mousedown="startWindowDrag"></div>
    <WindowControls class="absolute right-4 top-3" />
    <Card class="w-full max-w-xl p-8">
      <div class="mb-5 flex size-14 items-center justify-center rounded-2xl bg-destructive/10 text-destructive"><CircleHelp class="size-7" /></div>
      <p class="mb-2 text-sm font-medium text-destructive">独立卸载入口</p>
      <h1 class="text-2xl font-semibold">卸载 Lumo-Hub 并清理数据</h1>
      <p class="mt-4 text-sm leading-6 text-muted-foreground">此操作会删除 Lumo-Hub 的笔记、便签、倒数日、应用库记录和设置，然后启动 Windows 卸载程序。删除后无法恢复，请先导出备份。</p>
      <div class="mt-5 rounded-2xl border border-border bg-muted/50 p-4"><p class="mb-1 text-xs text-muted-foreground">将清理此数据目录</p><p class="break-all font-mono text-sm">{{ store.bootstrap.data_dir }}</p></div>
      <div class="mt-6 flex flex-wrap gap-3">
        <ConfirmDialog title="确认卸载并清理全部数据？" :description="`将永久删除：\n${store.bootstrap.data_dir}\n\n然后启动 Windows 卸载程序。此操作无法撤销。`" confirm-text="删除数据并卸载" destructive @confirm="runCleanUninstall">
          <template #trigger><Button variant="destructive">卸载并清理数据</Button></template>
        </ConfirmDialog>
        <Button variant="outline" @click="closeWindow">取消并关闭</Button>
      </div>
      <p v-if="store.message" role="status" class="mt-4 text-sm text-destructive">{{ store.message }}</p>
    </Card>
  </main>
  <main v-else-if="!store.settings.welcome_completed" class="relative flex min-h-screen items-center justify-center bg-background p-8">
    <div class="absolute inset-x-0 top-0 h-12" @mousedown="startWindowDrag"></div>
    <WindowControls class="absolute right-4 top-3" />
    <Card class="flex w-full max-w-2xl flex-col items-center p-12 text-center">
      <img src="/brand/lumohub.png" alt="LumoHub" class="mb-7 size-20 shrink-0 object-contain" />
      <p class="mb-3 text-sm font-medium text-primary">欢迎来到 Lumo-Hub</p>
      <h1 class="max-w-lg text-4xl font-semibold leading-tight tracking-tight">把日常的小事，<br />安放在一个地方。</h1>
      <p class="mt-5 max-w-md text-base leading-7 text-muted-foreground">记录想法、整理笔记、记住重要日子，也让常用的应用和文件触手可及。你的数据保存在本机。</p>
      <p class="mt-4 text-sm text-muted-foreground">按 {{ store.settings.launcher.shortcut }} 可随时唤出快捷面板，输入名称后按回车打开。</p><div class="mt-8 flex flex-wrap gap-3"><Button class="min-w-36" @click="store.completeWelcome">开始使用<ChevronRight data-icon="inline-end" /></Button><Button variant="outline" @click="completeWelcomeAndShowLauncher">试用快捷启动</Button></div>
      <p class="mt-8 text-xs text-muted-foreground">简洁、本地、属于你的工作台</p>
    </Card>
  </main>
  <div v-else class="flex h-dvh flex-col overflow-hidden bg-background">
      <header class="relative z-20 flex min-h-14 shrink-0 items-center gap-4 border-b border-border bg-background/90 px-6 py-2 backdrop-blur" @mousedown="startWindowDrag">
        <div class="flex min-w-0 flex-1 items-center gap-2">
          <div class="flex items-center pr-2"><img src="/brand/lumohub.png" alt="LumoHub" class="size-8 shrink-0 object-contain" /></div>
          <nav class="flex items-center gap-0.5" aria-label="快捷菜单">
            <DropdownMenu><DropdownMenuTrigger as-child><Button variant="ghost" class="h-8 px-2 text-sm">文件</Button></DropdownMenuTrigger><DropdownMenuContent align="start" class="w-52"><DropdownMenuItem @click="newItem"><Plus data-icon="inline-start"/>新建 / 快速记录<kbd class="ml-auto text-xs text-muted-foreground">Ctrl N</kbd></DropdownMenuItem><DropdownMenuItem @click="store.flushPendingSaves().then(()=>store.notify('已保存')).catch(e=>store.notify(String(e)))"><BookOpenText data-icon="inline-start"/>保存当前内容<kbd class="ml-auto text-xs text-muted-foreground">Ctrl S</kbd></DropdownMenuItem><DropdownMenuSeparator/><DropdownMenuItem @click="navigate('settings')"><Settings data-icon="inline-start"/>设置</DropdownMenuItem></DropdownMenuContent></DropdownMenu>
            <DropdownMenu><DropdownMenuTrigger as-child><Button variant="ghost" class="h-8 px-2 text-sm">导航</Button></DropdownMenuTrigger><DropdownMenuContent align="start" class="w-44"><DropdownMenuItem @click="navigate('home')"><Home data-icon="inline-start"/>首页</DropdownMenuItem><DropdownMenuItem @click="navigate('countdowns')"><CalendarPlus data-icon="inline-start"/>倒数日</DropdownMenuItem><DropdownMenuItem @click="navigate('stickies')"><StickyNoteIcon data-icon="inline-start"/>便签</DropdownMenuItem><DropdownMenuItem @click="navigate('notes')"><NotebookPen data-icon="inline-start"/>笔记</DropdownMenuItem><DropdownMenuItem @click="navigate('library')"><AppWindow data-icon="inline-start"/>应用库</DropdownMenuItem></DropdownMenuContent></DropdownMenu>
            <DropdownMenu><DropdownMenuTrigger as-child><Button variant="ghost" class="h-8 px-2 text-sm">帮助</Button></DropdownMenuTrigger><DropdownMenuContent align="start" class="w-44"><DropdownMenuItem @click="shortcutsOpen=true"><BookOpenText data-icon="inline-start"/>快捷键说明</DropdownMenuItem><DropdownMenuItem @click="navigate('settings')"><Settings data-icon="inline-start"/>设置</DropdownMenuItem><DropdownMenuItem @click="openAbout()"><CircleHelp data-icon="inline-start"/>关于</DropdownMenuItem><DropdownMenuItem @click="openAbout(true)"><RefreshCw data-icon="inline-start"/>检查更新</DropdownMenuItem></DropdownMenuContent></DropdownMenu>
          </nav>
        </div>
        <div class="relative flex w-full max-w-xl flex-1 items-center" data-no-drag>
          <Search class="pointer-events-none absolute left-3 size-4 text-muted-foreground" /><Input v-model="search" class="h-10 border-border bg-card pl-10 pr-10" placeholder="搜索应用、笔记、便签、倒数日…" aria-label="全局搜索" @focus="searchOpen=!!search.trim()" @keydown="handleSearchKeydown" />
          <div v-if="searchOpen&&search.trim()" class="absolute left-0 right-0 top-12 max-h-[70dvh] overflow-y-auto rounded-2xl border border-border bg-popover shadow-xl">
            <template v-for="(result,index) in searchResults" :key="`${result.kind}-${result.id}`"><p v-if="index===0||searchResults[index-1].kind!==result.kind" class="px-4 pt-3 pb-1 text-xs font-semibold text-muted-foreground">{{ {resource:'应用库',note:'笔记',sticky:'便签',countdown:'倒数日'}[result.kind] }}</p><Button variant="ghost" class="h-auto w-full items-start justify-start gap-3 rounded-none px-4 py-3 text-left whitespace-normal" :class="activeSearchIndex===index?'bg-accent':''" @mouseenter="activeSearchIndex=index" @click="openResult(result)"><span class="mt-0.5 rounded-lg bg-primary/10 px-2 py-1 text-[0.625rem] font-medium text-primary">{{ result.kind==='note'?'笔记':result.kind==='sticky'?'便签':result.kind==='countdown'?'倒数日':'应用库' }}</span><span class="min-w-0 flex-1"><span class="block truncate text-sm font-medium"><SearchHighlight :text="result.title" :query="search"/></span><span class="mt-1 block truncate text-xs text-muted-foreground"><SearchHighlight :text="result.subtitle" :query="search"/></span></span><ChevronRight class="mt-1 shrink-0 text-muted-foreground" /></Button></template>
            <div v-if="!searchResults.length" class="flex flex-col items-center gap-3 px-4 py-5"><p class="text-sm text-muted-foreground">没有找到相关内容</p><div class="flex gap-2"><Button size="sm" variant="outline" @click="showCapture(search);searchOpen=false">用此关键词创建记录</Button><Button size="sm" variant="outline" @click="addFromSearch()">添加启动项</Button></div></div>
            <div class="border-t border-border px-4 py-2 text-[0.6875rem] text-muted-foreground">本地搜索 · 搜索结果仅保存在此设备</div>
          </div>
        </div>
        <div class="flex min-w-0 flex-1 items-center justify-end gap-1"><Button variant="ghost" size="icon-sm" aria-label="刷新" title="刷新" @click="reloadPage"><RefreshCw /></Button><Button variant="ghost" size="icon-sm" aria-label="快捷键说明" title="快捷键说明" @click="shortcutsOpen=true"><BookOpenText/></Button><Button variant="ghost" size="icon-sm" aria-label="设置" title="设置" :aria-pressed="page==='settings'" @click="navigate('settings')"><Settings/></Button><WindowControls /></div>
      </header>
      <div class="flex min-h-0 flex-1 overflow-hidden">

    <aside class="relative flex shrink-0 flex-col border-r border-border bg-card/70" :class="sidebarDragWidth===null?'transition-[width] duration-200':'select-none'" :style="{width:sidebarCollapsed?'4.5rem':`${sidebarWidth}px`}">
      <div v-if="!sidebarCollapsed" role="separator" aria-label="调整侧边栏宽度" aria-orientation="vertical" :aria-valuenow="sidebarWidth" :aria-valuemin="100" :aria-valuemax="280" tabindex="0" title="拖动调整侧边栏宽度" class="absolute inset-y-0 -right-1 z-30 w-2 cursor-col-resize touch-none focus-visible:outline-none after:absolute after:inset-y-0 after:left-1/2 after:border-l after:border-dashed after:border-transparent hover:after:border-primary focus-visible:after:border-primary" :class="sidebarDragWidth!==null?'after:border-primary':''" @pointerdown="startSidebarResize" @pointermove="resizeSidebar" @pointerup="finishSidebarResize" @pointercancel="finishSidebarResize" @lostpointercapture="finishSidebarResize" @keydown="resizeSidebarWithKeyboard" />
      <nav class="flex flex-col gap-1 py-3" :class="sidebarCollapsed?'px-2':narrowSidebar?'px-1':'px-3'" aria-label="主导航">
        <Button v-for="item in nav" :key="item.id" variant="ghost" class="h-11 rounded-xl text-sm" :class="[sidebarCollapsed?'w-full justify-center px-0':narrowSidebar?'w-full justify-start gap-1 px-1':'w-full justify-start gap-3 px-3',page===item.id?'bg-primary/10 font-semibold text-primary':'text-muted-foreground']" :aria-label="sidebarCollapsed?item.label:undefined" :title="sidebarCollapsed?item.label:undefined" @click="navigate(item.id)"><component :is="item.icon" /><span v-if="!sidebarCollapsed" class="truncate">{{ item.label }}</span></Button>
      </nav>
      <div class="mt-auto flex shrink-0 flex-wrap items-center justify-center gap-1 border-t border-border p-2">
        <Button variant="ghost" :size="narrowSidebar&&!sidebarCollapsed?'icon-xs':'icon'" :aria-label="sidebarCollapsed?'展开侧边栏':'收起侧边栏'" @click="sidebarCollapsed=!sidebarCollapsed"><PanelLeftOpen v-if="sidebarCollapsed" /><PanelLeftClose v-else /></Button>
        <template v-if="!sidebarCollapsed">
        </template>
      </div>
    </aside>

    <section class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">

      <main :key="page" class="min-h-0 flex-1 px-4 pt-4" :class="page==='home'?'flex flex-col overflow-hidden':['notes','stickies','library','countdowns','settings'].includes(page)?'flex flex-col overflow-hidden pb-4':'overflow-y-auto overscroll-contain'">
        <component :is="currentView" v-bind="page==='settings'?{request:settingsRequest}:{}" @navigate="navigate" />
      </main>
    </section>
      </div>
    <div v-if="store.message" role="status" class="fixed bottom-6 right-6 z-50 max-w-lg rounded-2xl border border-border bg-popover px-4 py-3 text-sm shadow-lg">{{ store.message }}</div>
  </div>
</template>
