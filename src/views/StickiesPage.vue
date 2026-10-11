<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Plus, Search, StickyNote, Trash2, Pin, PinOff } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { ContextMenu, ContextMenuTrigger, ContextMenuContent, ContextMenuGroup, ContextMenuItem, ContextMenuSeparator } from '@/components/ui/context-menu'
import { api } from '@/lib/api'
import { useWorkspaceStore } from '@/stores/workspace'

const store=useWorkspaceStore()
const recentSticky=computed(()=>store.stickies.reduce<(typeof store.stickies)[number]|undefined>((latest,item)=>!latest||item.updated_at>latest.updated_at?item:latest,undefined))
const activeId=ref(recentSticky.value?.id ?? '')
const title=ref('')
const content=ref('')
const filter=ref('')
const filtered=computed(()=>store.stickies.filter(item=>`${item.title} ${item.content}`.toLocaleLowerCase().includes(filter.value.toLocaleLowerCase())).sort((a,b)=>Number(store.settings.product.pinned_stickies.includes(b.id))-Number(store.settings.product.pinned_stickies.includes(a.id))))
async function togglePin(id=activeId.value){const pinned=store.settings.product.pinned_stickies;try{await store.setSetting('product',{...store.settings.product,pinned_stickies:pinned.includes(id)?pinned.filter(itemId=>itemId!==id):[...pinned,id]})}catch(e){store.notify(String(e))}}
const deleteId=ref('')
const deleteOpen=ref(false)
const deleteTitle=computed(()=>store.stickies.find(item=>item.id===deleteId.value)?.title||'无标题便签')
function askDelete(id:string){deleteId.value=id;deleteOpen.value=true}
function formatDate(value:string){const date=new Date(value);return Number.isNaN(date.getTime())?value:date.toLocaleString('zh-CN',{year:'numeric',month:'2-digit',day:'2-digit',hour:'2-digit',minute:'2-digit'})}
const saveState=ref('')
let saveTimer:ReturnType<typeof setTimeout>|undefined
let unregisterSave:(()=>void)|undefined
let saveChain:Promise<void>=Promise.resolve()
let loadingRecord=false
const activeSticky=computed(()=>store.stickies.find(x=>x.id===activeId.value))

function load(itemId:string){loadingRecord=true;const item=store.stickies.find(x=>x.id===itemId);activeId.value=itemId;title.value=item?.title??'';content.value=item?.content??'';saveState.value='';loadingRecord=false}
async function saveNow():Promise<boolean>{
  if(saveTimer)clearTimeout(saveTimer)
  const id=activeId.value
  if(!id)return true
  if(saveState.value===''||saveState.value==='已保存'){await saveChain;return true}
  while(activeId.value===id){
    const savedTitle=title.value
    const savedContent=content.value
    saveState.value='保存中…'
    const job=saveChain.then(()=>api.updateSticky(id,savedTitle,savedContent))
    saveChain=job.catch(()=>{})
    try{
      await job
      const item=store.stickies.find(x=>x.id===id)
      if(item){item.title=savedTitle;item.content=savedContent;item.updated_at=new Date().toISOString()}
      if(activeId.value!==id)return true
      if(title.value===savedTitle&&content.value===savedContent){saveState.value='已保存';return true}
    }catch(e){
      if(activeId.value===id)saveState.value='保存失败'
      store.notify(`便签保存失败：${String(e)}`)
      return false
    }
  }
  return true
}
watch([title,content],()=>{if(loadingRecord||!activeId.value)return;saveState.value='未保存';if(saveTimer)clearTimeout(saveTimer);saveTimer=setTimeout(()=>void saveNow(),600)},{flush:'sync'})
async function select(id:string){if(id===activeId.value)return;if(!await saveNow())return;load(id)}
async function create(){if(!await saveNow())return;try{const item=await api.createSticky('新便签');await store.reload();load(item.id);store.notify('已新建便签')}catch(e){store.notify(String(e))}}
async function remove(id=activeId.value){const item=store.stickies.find(sticky=>sticky.id===id);if(!item)return;if(!await saveNow())return;try{await api.deleteSticky(item.id);await store.reload();if(id===activeId.value)load(recentSticky.value?.id??'');store.notify('便签已移入回收站')}catch(e){store.notify(String(e))}}
onMounted(()=>{unregisterSave=store.registerPendingSave(async()=>{if(!await saveNow())throw new Error('便签尚未保存')})})
onBeforeUnmount(()=>{if(saveTimer)clearTimeout(saveTimer);unregisterSave?.();if(activeId.value&&saveState.value==='未保存')void saveNow()})
watch(()=>store.stickies.length,(length)=>{if(!activeId.value&&length)load(recentSticky.value?.id??'')})
if(activeSticky.value)load(activeSticky.value.id)
watch(()=>store.searchTarget,target=>{if(target?.kind==='sticky'){load(target.id);store.searchTarget=null}},{immediate:true})
</script>

<template>
  <div class="grid h-full min-h-0 flex-1 grid-cols-[17rem_minmax(0,1fr)] gap-5">

      <Card class="flex min-h-0 flex-col gap-3 overflow-hidden p-3">
        <p class="shrink-0 px-1 text-base font-semibold">我的便签 · {{ store.stickies.length }}</p>
        <div class="flex shrink-0 items-center gap-2"><div class="relative min-w-0 flex-1"><Search class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" /><Input v-model="filter" class="pl-9" placeholder="搜索便签" aria-label="搜索便签" /></div><Button size="icon" aria-label="新建便签" title="新建便签" @click="create"><Plus /></Button></div>
        <div class="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto overscroll-contain">
        <ContextMenu v-for="item in filtered" :key="item.id"><ContextMenuTrigger as-child><Button variant="ghost" class="h-auto w-full flex-col items-stretch gap-1 rounded-xl px-3 py-3 text-left whitespace-normal" :class="activeId===item.id?'bg-accent':''" @click="select(item.id)"><span class="flex min-w-0 items-center gap-2 text-sm font-medium"><Pin v-if="store.settings.product.pinned_stickies.includes(item.id)" class="shrink-0 text-primary" aria-label="已置顶" /><span class="truncate">{{ item.title||'无标题便签' }}</span></span><span class="line-clamp-2 text-xs leading-5 text-muted-foreground">{{ item.content||'空白便签' }}</span></Button></ContextMenuTrigger><ContextMenuContent class="w-44"><ContextMenuGroup><ContextMenuItem @select="togglePin(item.id)"><PinOff v-if="store.settings.product.pinned_stickies.includes(item.id)" /><Pin v-else />{{ store.settings.product.pinned_stickies.includes(item.id)?'取消置顶':'置顶' }}</ContextMenuItem></ContextMenuGroup><ContextMenuSeparator /><ContextMenuGroup><ContextMenuItem variant="destructive" @select="askDelete(item.id)"><Trash2 />删除便签</ContextMenuItem></ContextMenuGroup></ContextMenuContent></ContextMenu>
        </div>
        <p v-if="!filtered.length" class="px-3 py-8 text-center text-sm text-muted-foreground">{{ filter?'没有匹配的便签':'创建便签开始记录' }}</p>
      </Card>
      <Card v-if="activeSticky" class="flex min-h-0 min-w-0 flex-col gap-4 overflow-hidden p-5">
        <div class="flex shrink-0 items-center gap-3"><Input v-model="title" aria-label="便签标题" class="min-w-0" placeholder="便签标题" /><Button :variant="store.settings.product.pinned_stickies.includes(activeId)?'secondary':'ghost'" size="icon" :aria-label="store.settings.product.pinned_stickies.includes(activeId)?'取消置顶':'置顶便签'" @click="togglePin()"><Pin /></Button><ConfirmDialog title="删除便签？" :description="`确认删除“${activeSticky.title || '无标题便签'}”？内容将移入回收站，可在设置中恢复。`" confirm-text="删除便签" destructive @confirm="remove()"><template #trigger><Button variant="danger" size="icon" aria-label="删除便签"><Trash2 /></Button></template></ConfirmDialog></div>
        <Textarea v-model="content" aria-label="便签内容" class="h-full min-h-0 flex-1 resize-none overflow-y-auto" placeholder="把想法写在这里……" />
        <div class="flex shrink-0 flex-wrap justify-end gap-x-5 gap-y-1 text-xs text-muted-foreground"><span>创建日期：{{ formatDate(activeSticky.created_at) }}</span><span>最近修改：{{ formatDate(activeSticky.updated_at) }}</span></div>
      </Card>
      <Card v-else class="flex flex-col items-center justify-center p-10 text-center"><StickyNote class="mb-4 text-primary" /><h2 class="font-semibold">选择一条便签</h2><p class="mt-2 text-sm text-muted-foreground">也可以创建一条新的便签。</p></Card>
    <ConfirmDialog v-model:open="deleteOpen" title="删除便签？" :description="`确认删除“${deleteTitle}”？内容将移入回收站，可在设置中恢复。`" confirm-text="删除便签" destructive @confirm="remove(deleteId)" />
  </div>
</template>
