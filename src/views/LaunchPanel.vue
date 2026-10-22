<script setup lang="ts">
import { builtinTools, openTool } from '@/lib/tools'
import { launcherOrder, fixedResourceIds } from '@/lib/launcher-preferences'
import SearchHighlight from '@/components/SearchHighlight.vue'
import { resourceImage } from '@/lib/library-preferences'
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { InputGroup, InputGroupAddon, InputGroupInput } from '@/components/ui/input-group'
import { computed, nextTick, onMounted, onUnmounted, ref, watch, type Component } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Globe, AppWindow, CalendarDays, FileText, NotebookPen, StickyNote, Monitor, Trash2, Folder, Terminal, SquareTerminal, Settings, Settings2, Calculator, Cpu, X, PanelsTopLeft } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import { api } from '@/lib/api'
import { applySettings, watchSystemTheme } from '@/lib/theme'
import { useWorkspaceStore } from '@/stores/workspace'
import type { SearchResult } from '@/lib/types'

type SystemLauncher={id:string;launcherId:string;title:string;aliases:string;subtitle:string;kind:'common';icon:Component}
type RecentPanelItem={id:string;resourceId:string;title:string;subtitle:string;kind:'recent'|'pinned'|'group'}
type PanelItem=SearchResult|{id:string;title:string;subtitle:string;kind:'action'}|SystemLauncher|RecentPanelItem
const searchScope=ref('resource')
const searchScopes=[{id:'resource',label:'应用库',icon:AppWindow},{id:'note',label:'笔记',icon:NotebookPen},{id:'sticky',label:'便签',icon:StickyNote},{id:'countdown',label:'日期',icon:CalendarDays},{id:'all',label:'全部内容',icon:Globe}]
const selectedScope=computed(()=>searchScopes.find(scope=>scope.id===searchScope.value)??searchScopes[0]!)
const systemLaunchers:SystemLauncher[]=[
  {id:'common:computer',launcherId:'computer',title:'此电脑',aliases:'我的电脑 this pc computer',subtitle:'系统启动项',kind:'common',icon:Monitor},
  {id:'common:recycle_bin',launcherId:'recycle_bin',title:'回收站',aliases:'recycle bin',subtitle:'系统启动项',kind:'common',icon:Trash2},
  {id:'common:explorer',launcherId:'explorer',title:'文件资源管理器',aliases:'explorer 资源管理器',subtitle:'系统启动项',kind:'common',icon:Folder},
  {id:'common:cmd',launcherId:'cmd',title:'CMD',aliases:'命令提示符 command',subtitle:'系统启动项',kind:'common',icon:SquareTerminal},
  {id:'common:powershell',launcherId:'powershell',title:'PowerShell',aliases:'终端',subtitle:'系统启动项',kind:'common',icon:Terminal},
  {id:'common:control_panel',launcherId:'control_panel',title:'控制面板',aliases:'control panel 系统设置',subtitle:'系统启动项',kind:'common',icon:Settings},
  {id:'common:notepad',launcherId:'notepad',title:'记事本',aliases:'notepad',subtitle:'系统启动项',kind:'common',icon:NotebookPen},
  {id:'common:calculator',launcherId:'calculator',title:'计算器',aliases:'calculator calc',subtitle:'系统启动项',kind:'common',icon:Calculator},
  {id:'common:task_manager',launcherId:'task_manager',title:'任务管理器',aliases:'task manager taskmgr 进程',subtitle:'系统启动项',kind:'common',icon:Cpu},
  {id:'common:windows_settings',launcherId:'windows_settings',title:'设置',aliases:'windows settings ms-settings 系统设置',subtitle:'系统启动项',kind:'common',icon:Settings2},
]
const actions=[{id:'sticky',title:'新建便签',subtitle:'随手记录一个想法',kind:'action' as const},{id:'note',title:'新建笔记',subtitle:'创建一篇长期记录',kind:'action' as const},{id:'countdown',title:'添加日期',subtitle:'记录一个重要日期',kind:'action' as const}]
const store=useWorkspaceStore()
const query=ref(''), results=ref<SearchResult[]>([]), selected=ref(0), error=ref(''), busy=ref(false), loading=ref(true)
const clockNow=ref(new Date())
const footerTimestamp=computed(()=>{const date=clockNow.value;const pad=(value:number)=>String(value).padStart(2,'0');return `${date.getFullYear()}-${pad(date.getMonth()+1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())} 周${['日','一','二','三','四','五','六'][date.getDay()]}`})
const resources=computed(()=>store.resources), notes=computed(()=>store.notes), stickies=computed(()=>store.stickies), countdowns=computed(()=>store.countdowns), settings=computed(()=>store.settings)
const commonIcons=ref<Record<string,string>>({})
let unlisten:UnlistenFn|undefined, changedListener:UnlistenFn|undefined, settingsListener:UnlistenFn|undefined, themeListener:(()=>void)|undefined
let timer:ReturnType<typeof setTimeout>|undefined, clockTimer:ReturnType<typeof setInterval>|undefined, generation=0, openingGeneration=0
let originalBodyMinWidth='', originalBodyOverflow='', originalRootOverflow=''
const sortedResources=computed(()=>[...resources.value].sort((a,b)=>Number(b.is_favorite)-Number(a.is_favorite)||b.usage_count-a.usage_count||(b.last_opened_at??'').localeCompare(a.last_opened_at??'')||a.name.localeCompare(b.name)))
const recentResources=computed(()=>resources.value.filter(item=>item.last_opened_at).sort((a,b)=>(b.last_opened_at??'').localeCompare(a.last_opened_at??'')).slice(0,6))
const pinnedGroup=computed(()=>store.groups.find(group=>group.id===settings.value?.client?.['lumo-launcher-pinned-group']))
const groupResources=computed(()=>pinnedGroup.value?sortedResources.value.filter(item=>item.group_id===pinnedGroup.value!.id):[])
const pinnedResources=computed(()=>fixedResourceIds(settings.value?.client).map(id=>resources.value.find(item=>item.id===id)).filter((item):item is NonNullable<typeof item>=>!!item))
const panelOrder=computed(()=>launcherOrder(settings.value?.client))
function sectionOrder(id:string){return panelOrder.value.indexOf(id)*10}

const enabledSystemLaunchers=computed(()=>systemLaunchers.filter(item=>(settings.value?.common_launchers??systemLaunchers.map(entry=>entry.launcherId)).includes(item.launcherId)))
const savedContent=computed<SearchResult[]>(()=>[
  ...notes.value.map(item=>({id:item.id,title:item.title||'无标题',subtitle:item.content,kind:'note' as const,updated_at:item.updated_at})),
  ...stickies.value.map(item=>({id:item.id,title:item.title||'无标题',subtitle:item.content,kind:'sticky' as const,updated_at:item.updated_at})),
  ...countdowns.value.map(item=>({id:item.id,title:item.title,subtitle:[item.target_date,item.description].filter(Boolean).join(' · '),kind:'countdown' as const,updated_at:item.updated_at})),
].sort((a,b)=>b.updated_at.localeCompare(a.updated_at)))
const items=computed<PanelItem[]>(()=>{
  if(!query.value.trim()){
    const resourceItems=sortedResources.value.map(item=>({id:item.id,title:item.name,subtitle:item.target,kind:'resource' as const}))
    const recentItems=searchScope.value==='resource'||searchScope.value==='all'?recentResources.value.map(item=>({id:`recent:${item.id}`,resourceId:item.id,title:item.name,subtitle:item.target,kind:'recent' as const})):[]
    const pinnedItems:RecentPanelItem[]=searchScope.value==='resource'||searchScope.value==='all'?pinnedResources.value.map(item=>({id:`pinned:${item.id}`,resourceId:item.id,title:item.name,subtitle:item.target,kind:'pinned'})):[]
    const groupItems:RecentPanelItem[]=searchScope.value==='resource'||searchScope.value==='all'?groupResources.value.map(item=>({id:`group:${item.id}`,resourceId:item.id,title:item.name,subtitle:item.target,kind:'group'})):[]
    const commonItems=searchScope.value==='resource'||searchScope.value==='all'?enabledSystemLaunchers.value:[]
    if(searchScope.value==='resource')return [...recentItems,...pinnedItems,...groupItems,...commonItems,...builtinTools.map(t=>({id:"tool:"+t.id,title:t.title,subtitle:t.description,kind:"resource" as const})),...resourceItems]
    return searchScope.value==='all'?[...recentItems,...pinnedItems,...groupItems,...commonItems,...builtinTools.map(t=>({id:"tool:"+t.id,title:t.title,subtitle:t.description,kind:"resource" as const})),...resourceItems,...savedContent.value]:savedContent.value.filter(item=>item.kind===searchScope.value)
  }
  const found=results.value.filter(item=>searchScope.value==='all'||item.kind===searchScope.value)
  const term=query.value.trim().toLocaleLowerCase()
  const commonMatches=searchScope.value==='resource'||searchScope.value==='all'?enabledSystemLaunchers.value.filter(item=>`${item.title} ${item.aliases}`.toLocaleLowerCase().includes(term)):[]
  const visible=[...found].sort((a,b)=>Number(b.kind==='resource')-Number(a.kind==='resource'))
  return [...commonMatches,...visible,...(searchScope.value==='all'?actions.filter(item=>(item.title+' 快速记录').includes(query.value.trim())):[])]
})
const toolItems=computed(()=>items.value.filter(item=>item.id.startsWith('tool:')))
const appItems=computed(()=>items.value.filter(item=>item.kind==='resource'&&!item.id.startsWith('tool:')))
const contentItems=computed(()=>items.value.filter(item=>item.kind!=='resource'&&item.kind!=='action'&&item.kind!=='common'&&item.kind!=='recent'&&item.kind!=='pinned'&&item.kind!=='group'))
const commonItems=computed(()=>items.value.filter((item):item is SystemLauncher=>item.kind==='common'))
const actionItems=computed(()=>items.value.filter(item=>item.kind==='action'))
const navigableItems=computed(()=>[...items.value].sort((a,b)=>{
  const rank=(item:PanelItem)=>item.kind==='group'?sectionOrder('resource')-1:sectionOrder(item.kind==='resource'?'resource':item.kind==='recent'?'recent':item.kind==='pinned'?'pinned':item.kind==='common'?'common':'resource')
  return rank(a)-rank(b)||Number(b.id.startsWith('tool:'))-Number(a.id.startsWith('tool:'))
}))
const selectedId=computed(()=>navigableItems.value[selected.value]?.id)
function resource(id:string){return resources.value.find(item=>item.id===id)}
function resourceIcon(id:string){const tool=builtinTools.find(t=>'tool:'+t.id===id);if(tool)return tool.icon;const kind=resource(id)?.kind;return kind==='web'?Globe:kind==='file'?FileText:AppWindow}
function icon(kind:string){return kind==='note'?NotebookPen:kind==='sticky'?StickyNote:kind==='countdown'?CalendarDays:FileText}
function itemIcon(item:PanelItem){if(item.kind==='common')return item.icon;if(item.kind==='action')return item.id==='sticky'?StickyNote:item.id==='note'?NotebookPen:CalendarDays;return icon(item.kind)}
async function refresh(){
  const version=++generation
  if(!query.value.trim()){results.value=[];selected.value=0;return}
  try{const found=await api.search(query.value);if(version===generation){results.value=found.slice(0,100);selected.value=0}}
  catch(e){if(version===generation)error.value=String(e)}
}
async function opened(){
  const currentOpening=++openingGeneration;query.value='';error.value='';loading.value=true
  await nextTick();document.querySelector<HTMLInputElement>('#quick-search')?.focus()
  try{
    const current=await api.bootstrap();if(currentOpening!==openingGeneration)return
    applySettings(current.settings)
    if(current.recovery_error)throw new Error(current.recovery_error)
    const [icons]=await Promise.all([api.commonLauncherIcons(),store.reload()])
    if(currentOpening!==openingGeneration)return
    commonIcons.value=icons;await refresh()
  }catch(e){error.value=String(e)}finally{if(currentOpening===openingGeneration)loading.value=false}
}
watch(searchScope,()=>{selected.value=0})
watch(panelOrder,()=>{selected.value=0})
watch(items,()=>{selected.value=Math.min(selected.value,Math.max(0,items.value.length-1))})
watch(query,()=>{if(timer)clearTimeout(timer);results.value=[];++generation;timer=setTimeout(()=>{timer=undefined;void refresh()},100)})
async function activate(item:PanelItem){
  if(item.kind==='action'){await main(item.id==='note'?'notes':item.id==='sticky'?'stickies':'countdowns');return}
  if(busy.value)return;busy.value=true
  try{if(item.id.startsWith('tool:')){await openTool(item.id.slice(5));await api.hideLauncher()}else if(item.kind==='common'){await api.launchCommon(item.launcherId);await api.hideLauncher()}else if(item.kind==='resource'||item.kind==='recent'||item.kind==='pinned'||item.kind==='group'){await api.launchResource(item.kind==='recent'||item.kind==='pinned'||item.kind==='group'?item.resourceId:item.id);await api.hideLauncher()}else await api.openMainPage(item.kind==='note'?'notes':item.kind==='sticky'?'stickies':'countdowns',item.id)}
  catch(e){error.value=String(e)}finally{busy.value=false}
}
async function main(page='home',id:string|null=null){try{await api.openMainPage(page,id)}catch(e){error.value=String(e)}}
function selectFocused(event:FocusEvent){
  const target=(event.target as HTMLElement).closest<HTMLElement>('[data-launch-grid] [role="option"]')
  if(!target)return
  const index=navigableItems.value.findIndex(item=>`quick-result-${item.id}`===target.id)
  if(index>=0)selected.value=index
}
async function openSelected(event:KeyboardEvent){
  if(event.isComposing||event.repeat||event.ctrlKey||event.altKey||event.metaKey||event.shiftKey)return
  const target=event.target as HTMLElement
  if(target.closest('[data-slot="select-trigger"], [data-slot="select-content"], button:not([role="option"]), a, [role="button"]'))return
  event.preventDefault();event.stopPropagation()
  if(timer&&query.value.trim()){clearTimeout(timer);timer=undefined;await refresh()}
  const item=navigableItems.value[selected.value]
  if(item)await activate(item)
}
function keydown(event:KeyboardEvent){
  if(event.isComposing||event.defaultPrevented)return
  if(event.ctrlKey&&event.key.toLowerCase()==='k'){event.preventDefault();document.querySelector<HTMLInputElement>('#quick-search')?.focus();return}
  if(event.ctrlKey&&event.key.toLowerCase()==='n'){event.preventDefault();void api.openMainPage('stickies');return}
  if(event.key==='F1'){event.preventDefault();return}
  if(event.key==='Escape'){event.preventDefault();void api.hideLauncher();return}
  if(!navigableItems.value.length){if(['ArrowDown','ArrowUp','ArrowLeft','ArrowRight'].includes(event.key)){event.preventDefault();document.querySelector<HTMLInputElement>('#quick-search')?.focus()}return}
  if(['ArrowDown','ArrowUp','ArrowLeft','ArrowRight'].includes(event.key)){
    event.preventDefault()
    const nodes=[...document.querySelectorAll<HTMLElement>('[data-launch-grid] [role="option"]')]
    const current=nodes.find(node=>node.id===`quick-result-${selectedId.value}`)
    if(current){
      const rect=current.getBoundingClientRect()
      const cx=rect.left+rect.width/2, cy=rect.top+rect.height/2
      const horizontal=event.key==='ArrowLeft'||event.key==='ArrowRight'
      const forward=event.key==='ArrowRight'||event.key==='ArrowDown'
      const candidates=nodes.filter(node=>node!==current).map(node=>{
        const box=node.getBoundingClientRect()
        const dx=box.left+box.width/2-cx, dy=box.top+box.height/2-cy
        return {node,primary:horizontal?dx:dy,secondary:horizontal?dy:dx}
      }).filter(entry=>(forward?entry.primary>2:entry.primary< -2)&&(!horizontal||Math.abs(entry.secondary)<rect.height/2))
      candidates.sort((a,b)=>Math.abs(a.primary)-Math.abs(b.primary)||Math.abs(a.secondary)-Math.abs(b.secondary))
      const next=candidates[0]?.node
      if(next){const index=navigableItems.value.findIndex(item=>`quick-result-${item.id}`===next.id);if(index>=0)selected.value=index}
    }
    void nextTick(()=>{const id=navigableItems.value[selected.value]?.id;if(id)document.getElementById(`quick-result-${id}`)?.scrollIntoView({block:'nearest'})})
  }
}
function startDrag(event:MouseEvent){if(event.button===0&&!(event.target as HTMLElement).closest('input,button,[role=combobox]'))void getCurrentWindow().startDragging().catch(()=>{})}
onMounted(async()=>{
  clockTimer=setInterval(()=>{clockNow.value=new Date()},1000)
  originalBodyMinWidth=document.body.style.minWidth;originalBodyOverflow=document.body.style.overflow;originalRootOverflow=document.documentElement.style.overflow
  document.body.style.minWidth='0';document.body.style.overflow='hidden';document.documentElement.style.overflow='hidden'
  unlisten=await listen('quick-launch-open',opened)
  changedListener=await listen('lumo-resources-changed',async()=>{try{await store.reload()}catch{}})
  window.addEventListener('keydown',keydown)
  settingsListener=await listen('lumo-settings-changed',async()=>{try{await store.reload()}catch{}})
  await opened();themeListener=watchSystemTheme(()=>settings.value!)
})
onUnmounted(()=>{window.removeEventListener('keydown',keydown);unlisten?.();changedListener?.();settingsListener?.();themeListener?.();if(timer)clearTimeout(timer);if(clockTimer)clearInterval(clockTimer);document.body.style.minWidth=originalBodyMinWidth;document.body.style.overflow=originalBodyOverflow;document.documentElement.style.overflow=originalRootOverflow})
</script>

<template>
  <div class="launch-panel flex h-dvh w-full min-w-0 flex-col overflow-hidden bg-background" @keydown.enter.capture="openSelected" @focusin="selectFocused">
    <div class="shrink-0 border-b border-border p-2" @mousedown="startDrag">
      <InputGroup class="h-10 w-full">
        <InputGroupAddon align="inline-start">
          <Select v-model="searchScope">
            <SelectTrigger class="launch-scope" aria-label="搜索范围" @keydown.stop><SelectValue><component :is="selectedScope.icon"/>{{ selectedScope.label }}</SelectValue></SelectTrigger>
            <SelectContent><SelectGroup><SelectItem v-for="scope in searchScopes" :key="scope.id" :value="scope.id"><component :is="scope.icon"/>{{ scope.label }}</SelectItem></SelectGroup></SelectContent>
          </Select>
        </InputGroupAddon>
        <InputGroupInput id="quick-search" v-model="query" :placeholder="searchScope==='all'?'搜索应用、笔记、便签、日期或拼音别名…':searchScope==='resource'?'搜索启动项、拼音首字母或别名…':`搜索${selectedScope.label}、拼音或别名…`" autocomplete="off" role="combobox" aria-label="搜索内容" aria-controls="quick-results" :aria-expanded="true" :aria-activedescendant="selectedId?`quick-result-${selectedId}`:undefined"/>
      </InputGroup>
    </div>
    <div class="launch-content flex min-h-0 min-w-0 flex-1 flex-col gap-6 overflow-x-hidden overflow-y-auto overscroll-contain p-5">
      <p v-if="error" class="text-sm text-destructive" role="alert">{{ error }}</p>
      <p v-if="loading" class="text-sm text-muted-foreground">正在读取本地内容…</p>
      <template v-for="section in panelOrder" :key="section">
      <section :style="{order:sectionOrder('recent')}" v-if="section==='recent'&&!loading&&!query.trim()&&recentResources.length&&['resource','all'].includes(searchScope)" class="flex flex-col gap-2"><h2 class="text-xs font-medium text-muted-foreground">最近使用打开的应用 · {{ recentResources.length }}</h2><div data-launch-grid="recent" class="grid min-w-0 grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-1" role="listbox" aria-label="最近使用"><Button v-for="item in recentResources" :id="`quick-result-recent:${item.id}`" :key="item.id" variant="ghost" class="h-28 w-full min-w-0 flex-col items-center justify-center gap-2 rounded-2xl border border-transparent bg-transparent px-2 py-2 text-foreground shadow-none transition-colors hover:border-transparent hover:bg-muted/70 focus-visible:border-transparent" :class="selectedId===`recent:${item.id}`?'launch-selected':''" :disabled="busy" :title="item.name" role="option" :aria-selected="selectedId===`recent:${item.id}`" @click="activate({id:`recent:${item.id}`,resourceId:item.id,title:item.name,subtitle:item.target,kind:'recent'})"><span class="relative flex size-10 shrink-0 items-center justify-center"><img v-if="resourceImage(settings?.client,item)" :src="resourceImage(settings?.client,item)!" alt="" class="size-10 shrink-0 object-contain"/><component :is="resourceIcon(item.id)" v-else class="size-10 text-primary"/></span><span class="w-full truncate text-center text-sm">{{ item.name }}</span></Button></div></section>
      <section :style="{order:sectionOrder('pinned')}" v-if="section==='pinned'&&!loading&&!query.trim()&&pinnedResources.length&&['resource','all'].includes(searchScope)" class="flex flex-col gap-2"><h2 class="text-xs font-medium text-muted-foreground">固定项 · {{ pinnedResources.length }}</h2><div data-launch-grid="pinned" class="grid min-w-0 grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-1" role="listbox" aria-label="固定项"><Button v-for="item in pinnedResources" :id="`quick-result-pinned:${item.id}`" :key="item.id" variant="ghost" class="h-28 w-full min-w-0 flex-col items-center justify-center gap-2 rounded-2xl border border-transparent bg-transparent px-2 py-2 text-foreground shadow-none transition-colors hover:border-transparent hover:bg-muted/70 focus-visible:border-transparent" :class="selectedId===`pinned:${item.id}`?'launch-selected':''" :disabled="busy" :title="item.name" role="option" :aria-selected="selectedId===`pinned:${item.id}`" @click="activate({id:`pinned:${item.id}`,resourceId:item.id,title:item.name,subtitle:item.target,kind:'pinned'})"><span class="relative flex size-10 shrink-0 items-center justify-center"><img v-if="resourceImage(settings?.client,item)" :src="resourceImage(settings?.client,item)!" alt="" class="size-10 shrink-0 object-contain"/><component :is="resourceIcon(item.id)" v-else class="size-10 text-primary"/></span><span class="w-full truncate text-center text-sm">{{ item.name }}</span></Button></div></section>
      <section :style="{order:sectionOrder('resource')-1}" v-if="section==='resource'&&!loading&&!query.trim()&&pinnedGroup&&['resource','all'].includes(searchScope)" class="flex flex-col gap-2"><h2 class="text-xs font-medium text-muted-foreground">{{ pinnedGroup?.name }} · {{ groupResources.length }}</h2><div data-launch-grid="group" class="grid min-w-0 grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-1" role="listbox" aria-label="固定分组"><p v-if="!groupResources.length" class="col-span-full text-sm text-muted-foreground">此分组暂无启动项</p><Button v-for="item in groupResources" :id="`quick-result-group:${item.id}`" :key="item.id" variant="ghost" class="h-28 w-full min-w-0 flex-col items-center justify-center gap-2 rounded-2xl border border-transparent bg-transparent px-2 py-2 text-foreground shadow-none transition-colors hover:border-transparent hover:bg-muted/70 focus-visible:border-transparent" :class="selectedId===`group:${item.id}`?'launch-selected':''" :disabled="busy" :title="item.name" role="option" :aria-selected="selectedId===`group:${item.id}`" @click="activate({id:`group:${item.id}`,resourceId:item.id,title:item.name,subtitle:item.target,kind:'group'})"><span class="relative flex size-10 shrink-0 items-center justify-center"><img v-if="resourceImage(settings?.client,item)" :src="resourceImage(settings?.client,item)!" alt="" class="size-10 shrink-0 object-contain"/><component :is="resourceIcon(item.id)" v-else class="size-10 text-primary"/></span><span class="w-full truncate text-center text-sm">{{ item.name }}</span></Button></div></section>
      <section :style="{order:sectionOrder('common')}" v-if="section==='common'&&!loading&&commonItems.length" class="flex flex-col gap-2"><h2 class="text-xs font-medium text-muted-foreground">系统启动项 · {{ commonItems.length }}</h2><div data-launch-grid="common" class="grid min-w-0 grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-1" role="listbox" aria-label="系统启动项"><Button v-for="item in commonItems" :id="`quick-result-${item.id}`" :key="item.id" variant="ghost" class="h-28 w-full min-w-0 flex-col items-center justify-center gap-2 rounded-2xl border border-transparent bg-transparent px-2 py-2 text-foreground shadow-none transition-colors hover:border-transparent hover:bg-muted/70 focus-visible:border-transparent" :class="selectedId===item.id?'launch-selected':''" :title="item.title" :disabled="busy" role="option" :aria-selected="selectedId===item.id" @click="activate(item)"><img v-if="commonIcons[item.launcherId]" :src="commonIcons[item.launcherId]" alt="" class="size-10 shrink-0 object-contain"/><component :is="item.icon" v-else class="size-10 shrink-0 text-primary"/><span class="w-full truncate text-center text-sm">{{ item.title }}</span></Button></div></section>
      <section v-if="section==='resource'&&!loading&&toolItems.length" :style="{order:sectionOrder('resource')}" class="flex flex-col gap-2"><h2 class="text-xs font-medium text-muted-foreground">内置工具 · {{toolItems.length}}</h2><div data-launch-grid="tools" class="grid min-w-0 grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-1" role="listbox" aria-label="内置工具"><Button v-for="item in toolItems" :key="item.id" :id="`quick-result-${item.id}`" variant="ghost" class="h-28 w-full min-w-0 flex-col gap-2 rounded-2xl px-2" :class="selectedId===item.id?'launch-selected':''" role="option" :aria-selected="selectedId===item.id" :title="item.subtitle" :disabled="busy" @click="activate(item)"><component :is="resourceIcon(item.id)" class="size-10 text-primary"/><span class="w-full truncate text-center text-sm"><SearchHighlight :text="item.title" :query="query"/></span></Button></div></section>
      <section :style="{order:sectionOrder('resource')}" v-if="section==='resource'&&!loading&&['resource','all'].includes(searchScope)&&appItems.length" class="flex flex-col gap-2"><h2 class="text-xs font-medium text-muted-foreground">{{ query.trim()?'匹配的应用':'全部应用' }} · {{ appItems.length }}</h2><div id="quick-results" data-launch-grid="resource" class="grid min-w-0 grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-1" role="listbox" aria-label="应用库"><Button v-for="item in appItems" :id="`quick-result-${item.id}`" :key="item.id" variant="ghost" class="h-28 w-full min-w-0 flex-col items-center justify-center gap-2 rounded-2xl border border-transparent bg-transparent px-2 py-2 text-foreground shadow-none transition-colors hover:border-transparent hover:bg-muted/70 focus-visible:border-transparent" :class="selectedId===item.id?'launch-selected':''" :title="item.title" :disabled="busy" role="option" :aria-selected="selectedId===item.id" @click="activate(item)"><span class="relative flex size-10 shrink-0 items-center justify-center"><img v-if="resource(item.id)&&resourceImage(settings?.client,resource(item.id)!)" :src="resourceImage(settings?.client,resource(item.id)!)!" alt="" class="size-10 shrink-0 object-contain"/><component :is="resourceIcon(item.id)" v-else class="size-10 text-primary"/></span><span class="w-full truncate text-center text-sm"><SearchHighlight :text="item.title" :query="query"/></span></Button></div></section>
      </template>
      <section :style="{order:50}" v-if="!loading&&contentItems.length" class="flex flex-col gap-2"><h2 class="text-xs font-medium text-muted-foreground">{{ query.trim()||searchScope==='all'?'匹配的内容':selectedScope.label }} · {{ contentItems.length }}</h2><div data-launch-grid="content" class="grid min-w-0 grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-1" role="listbox" aria-label="记录"><Button v-for="item in contentItems" :id="`quick-result-${item.id}`" :key="item.id" variant="ghost" class="h-28 w-full min-w-0 flex-col items-center justify-center gap-2 rounded-2xl border border-transparent bg-transparent px-2 py-2 text-foreground shadow-none transition-colors hover:border-transparent hover:bg-muted/70 focus-visible:border-transparent" :class="selectedId===item.id?'launch-selected':''" :title="item.title" role="option" :aria-selected="selectedId===item.id" @click="activate(item)"><component :is="icon(item.kind)" class="size-10 shrink-0 text-primary"/><span class="w-full truncate text-center text-sm"><SearchHighlight :text="item.title" :query="query"/></span></Button></div></section>
      <section :style="{order:60}" v-if="!loading&&actionItems.length" class="flex flex-col gap-3"><h2 class="text-xs font-medium text-muted-foreground">快速记录 · {{ actionItems.length }}</h2><div data-launch-grid="action" role="listbox" class="grid min-w-0 grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-2"><Button v-for="item in actionItems" :id="`quick-result-${item.id}`" :key="item.id" role="option" :aria-selected="selectedId===item.id" variant="ghost" class="h-28 w-full min-w-0 flex-col items-center justify-center gap-3 rounded-2xl px-3 py-3 has-[>svg]:px-3" :class="selectedId===item.id?'launch-selected':''" @click="activate(item)"><component :is="itemIcon(item)" class="size-10 text-primary"/><span class="w-full truncate text-center text-sm">{{ item.title }}</span></Button></div></section>
      <div v-if="!loading&&query.trim()&&!items.length" class="grid min-w-0 grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-3"><Button variant="outline" class="h-20" @click="main('library')">添加启动项 / 扫描本机</Button></div>
    </div>
    <footer class="flex h-11 shrink-0 items-center justify-between gap-3 border-t border-border bg-background px-4" aria-label="快捷启动窗口操作">
      <time class="truncate text-sm text-muted-foreground" :datetime="clockNow.toISOString()">{{ footerTimestamp }}</time>
      <div class="flex shrink-0 items-center gap-1">
        <Button variant="ghost" size="icon" aria-label="关闭快捷启动" title="关闭快捷启动" @click="api.hideLauncher()"><X class="size-4"/></Button>
        <Button variant="ghost" size="icon" aria-label="打开主面板" title="打开主面板" @click="main('home')"><PanelsTopLeft class="size-4"/></Button>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.launch-scope {
  border: 0;
  box-shadow: none;
  background: transparent;
}
.launch-panel :deep([role="option"]) {
  border: 0;
  outline: none;
  box-shadow: none;
}
.launch-panel :deep([role="option"]:is(:hover, :focus-visible, .launch-selected)) {
  background: var(--muted);
  box-shadow: none;
}

.launch-content {
  scrollbar-width: none;
}

.launch-content::-webkit-scrollbar {
  display: none;
}
</style>
