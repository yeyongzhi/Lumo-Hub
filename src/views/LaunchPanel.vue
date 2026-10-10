<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Search, Globe, AppWindow, CalendarDays, FileText, LayoutDashboard, NotebookPen, StickyNote, X } from '@lucide/vue'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { api } from '@/lib/api'
import { applySettings, watchSystemTheme } from '@/lib/theme'
import type { Resource, SearchResult, AppSettings, Note, Sticky } from '@/lib/types'

const query=ref(''), results=ref<SearchResult[]>([]), selected=ref(0), error=ref(''), busy=ref(false), loading=ref(true)
const resources=ref<Resource[]>([]), notes=ref<Note[]>([]), stickies=ref<Sticky[]>([]), settings=ref<AppSettings>()
const minimal=computed(()=>settings.value?.launcher.minimal_mode??false)
let unlisten:UnlistenFn|undefined, changedListener:UnlistenFn|undefined, themeListener:(()=>void)|undefined
let timer:ReturnType<typeof setTimeout>|undefined, generation=0, openingGeneration=0
const sortedResources=computed(()=>[...resources.value].sort((a,b)=>Number(b.is_favorite)-Number(a.is_favorite)||b.usage_count-a.usage_count||(b.last_opened_at??'').localeCompare(a.last_opened_at??'')||a.name.localeCompare(b.name)))
const recentResources=computed(()=>resources.value.filter(item=>item.last_opened_at).sort((a,b)=>(b.last_opened_at??'').localeCompare(a.last_opened_at??'')).slice(0,6))
const recentContent=computed(()=>[
  ...notes.value.map(item=>({...item,page:'notes',icon:NotebookPen})),
  ...stickies.value.map(item=>({...item,page:'stickies',icon:StickyNote})),
].sort((a,b)=>b.updated_at.localeCompare(a.updated_at)).slice(0,6))
const items=computed(()=>query.value.trim()?results.value:(minimal.value?sortedResources.value:sortedResources.value.slice(0,50)).map(item=>({id:item.id,title:item.name,subtitle:item.target,kind:'resource' as const})))
const appItems=computed(()=>items.value.filter(item=>item.kind==='resource'))
const contentItems=computed(()=>items.value.filter(item=>item.kind!=='resource'))
const selectedId=computed(()=>items.value[selected.value]?.id)
function resource(id:string){return resources.value.find(item=>item.id===id)}
function resourceIcon(id:string){const kind=resource(id)?.kind;return kind==='web'?Globe:kind==='file'?FileText:AppWindow}
function icon(kind:string){return kind==='note'?NotebookPen:kind==='countdown'?CalendarDays:FileText}
async function refresh(){
  const version=++generation
  if(!query.value.trim()){results.value=[];selected.value=0;return}
  try{const found=await (minimal.value?api.search(query.value):api.searchResources(query.value));if(version===generation){results.value=minimal.value?found:found.slice(0,50);selected.value=0}}
  catch(e){if(version===generation)error.value=String(e)}
}
async function opened(){
  const currentOpening=++openingGeneration;query.value='';error.value='';loading.value=true
  await nextTick();document.querySelector<HTMLInputElement>('#quick-search')?.focus()
  try{
    const current=await api.bootstrap();if(currentOpening!==openingGeneration)return
    settings.value=current.settings;applySettings(current.settings)
    if(current.recovery_error)throw new Error(current.recovery_error)
    const [r,n,s]=await Promise.all([api.listResources(),api.listNotes(),api.listStickies()])
    if(currentOpening!==openingGeneration)return
    resources.value=r;notes.value=n;stickies.value=s;await refresh()
  }catch(e){error.value=String(e)}finally{if(currentOpening===openingGeneration)loading.value=false}
}
watch(query,()=>{if(timer)clearTimeout(timer);results.value=[];++generation;timer=setTimeout(refresh,100)})
async function activate(item:SearchResult){
  if(busy.value)return;busy.value=true
  try{if(item.kind==='resource'){await api.launchResource(item.id);await api.hideLauncher()}else await api.openMainPage(item.kind==='note'?'notes':'countdowns',item.id)}
  catch(e){error.value=String(e)}finally{busy.value=false}
}
async function main(page='home',id:string|null=null){try{await api.openMainPage(page,id)}catch(e){error.value=String(e)}}
function keydown(event:KeyboardEvent){
  if(event.isComposing)return
  if(event.key==='Escape'){event.preventDefault();void api.hideLauncher();return}
  if((event.target as HTMLElement).closest('button')&&event.key==='Enter')return
  if(!items.value.length)return
  if(['ArrowDown','ArrowUp','ArrowLeft','ArrowRight'].includes(event.key)){
    event.preventDefault();selected.value=(selected.value+(['ArrowDown','ArrowRight'].includes(event.key)?1:items.value.length-1))%items.value.length
    void nextTick(()=>document.getElementById(`quick-result-${selectedId.value}`)?.scrollIntoView({block:'nearest'}))
  }
  if(event.key==='Enter'){event.preventDefault();const item=items.value[selected.value];if(item)void activate(item)}
}
function startDrag(event:MouseEvent){if(event.button===0&&!(event.target as HTMLElement).closest('input,button'))void getCurrentWindow().startDragging().catch(()=>{})}
onMounted(async()=>{
  unlisten=await listen('quick-launch-open',opened)
  changedListener=await listen('lumo-resources-changed',async()=>{try{resources.value=await api.listResources()}catch{}})
  await opened();themeListener=watchSystemTheme(()=>settings.value!)
})
onUnmounted(()=>{unlisten?.();changedListener?.();themeListener?.();if(timer)clearTimeout(timer)})
</script>

<template>
  <div class="flex h-dvh flex-col overflow-hidden bg-background" @keydown="keydown">
    <div class="flex shrink-0 items-center gap-3 border-b border-border px-6 py-4" @mousedown="startDrag">
      <Search class="size-5 shrink-0 text-primary"/><Input id="quick-search" v-model="query" class="h-12" :placeholder="minimal?'搜索应用、笔记、日期或拼音别名…':'搜索应用、拼音首字母或别名…'" autocomplete="off" role="combobox" aria-label="搜索内容" aria-controls="quick-results" :aria-expanded="true" :aria-activedescendant="selectedId?`quick-result-${selectedId}`:undefined"/>
      <Button variant="ghost" size="icon" title="打开主界面" aria-label="打开主界面" @click="main()"><LayoutDashboard/></Button><Button variant="ghost" size="icon" aria-label="收起面板" @click="api.hideLauncher()"><X/></Button>
    </div>
    <div class="flex min-h-0 flex-1 flex-col gap-8 overflow-y-auto overscroll-contain p-6">
      <p v-if="error" class="text-sm text-destructive" role="alert">{{ error }}</p>
      <p v-if="loading" class="text-sm text-muted-foreground">正在读取本地内容…</p>
      <template v-else-if="minimal">
        <Card v-if="!query.trim()"><CardHeader><CardTitle class="text-sm">最近使用</CardTitle></CardHeader><CardContent class="flex flex-col gap-4">
          <div v-if="recentResources.length" class="flex flex-wrap gap-2"><Button v-for="item in recentResources" :key="item.id" variant="outline" :disabled="busy" @click="activate({id:item.id,title:item.name,subtitle:item.target,kind:'resource'})"><img v-if="item.icon_data" :src="item.icon_data" alt="" class="size-5 object-contain"/><AppWindow v-else data-icon="inline-start"/><span class="max-w-40 truncate">{{ item.name }}</span></Button></div>
          <div v-if="recentContent.length" class="flex flex-wrap gap-2"><Button v-for="item in recentContent" :key="item.id" variant="ghost" size="sm" @click="main(item.page,item.id)"><component :is="item.icon" data-icon="inline-start"/><span class="max-w-40 truncate">{{ item.title||'无标题' }}</span></Button></div>
          <p v-if="!recentResources.length&&!recentContent.length" class="text-sm text-muted-foreground">打开应用或编辑内容后，会显示在这里。</p>
        </CardContent></Card>
        <section><h2 class="mb-4 text-sm font-medium text-muted-foreground">{{ query.trim()?'匹配的启动项':'全部应用库' }}</h2>
          <div id="quick-results" class="grid grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-4" role="listbox" aria-label="应用库">
            <Button v-for="item in appItems" :id="`quick-result-${item.id}`" :key="item.id" variant="ghost" class="h-28 min-w-0 flex-col gap-3 rounded-2xl" :class="query.trim()&&selectedId===item.id?'bg-accent':''" :disabled="busy" role="option" :aria-selected="selectedId===item.id" @click="activate(item)"><img v-if="resource(item.id)?.icon_data" :src="resource(item.id)!.icon_data!" alt="" class="size-10 object-contain"/><component :is="resourceIcon(item.id)" v-else class="size-10 text-primary"/><span class="w-full truncate text-center">{{ item.title }}</span></Button>
          </div><p v-if="!appItems.length" class="py-4 text-sm text-muted-foreground">{{ query.trim()?'没有匹配的启动项。':'应用库为空，点击右上角主界面图标添加应用。' }}</p>
        </section>
        <section v-if="contentItems.length" class="flex flex-col gap-2"><h2 class="text-sm font-medium text-muted-foreground">笔记与倒数日</h2><Button v-for="item in contentItems" :id="`quick-result-${item.id}`" :key="item.id" variant="ghost" class="h-auto justify-start gap-3 p-3" :class="selectedId===item.id?'bg-accent':''" @click="activate(item)"><component :is="icon(item.kind)" data-icon="inline-start"/><span class="flex min-w-0 flex-1 flex-col gap-1 text-left"><span class="truncate">{{ item.title }}</span><span class="truncate text-xs text-muted-foreground">{{ item.subtitle }}</span></span></Button></section>
      </template>
      <div v-else id="quick-results" class="flex flex-col gap-1" role="listbox" aria-label="启动项结果">
        <Button v-for="(item,index) in items.slice(0,50)" :id="`quick-result-${item.id}`" :key="item.id" variant="ghost" class="h-auto shrink-0 justify-start gap-3 px-3 py-3" :class="selected===index?'bg-accent':''" :disabled="busy" role="option" :aria-selected="selected===index" @click="activate(item)" @pointermove="selected=index"><img v-if="resource(item.id)?.icon_data" :src="resource(item.id)!.icon_data!" alt="" class="size-8 shrink-0 object-contain"/><AppWindow v-else/><span class="flex min-w-0 flex-1 flex-col items-start gap-1"><span class="truncate">{{ item.title }}</span><span class="max-w-full truncate text-xs text-muted-foreground">{{ item.subtitle }}</span></span></Button>
        <p v-if="!items.length&&!error" class="py-8 text-center text-sm text-muted-foreground">{{ query?'没有匹配的启动项':'先在应用库添加启动项，即可快速打开。' }}</p>
      </div>
    </div>
    <p class="shrink-0 px-6 pb-3 text-xs text-muted-foreground">方向键选择 · Enter 打开 · Esc 收起</p>
  </div>
</template>
