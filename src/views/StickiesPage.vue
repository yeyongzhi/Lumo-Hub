<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Check, Plus, StickyNote, Trash2 } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { api } from '@/lib/api'
import { useWorkspaceStore } from '@/stores/workspace'

const store=useWorkspaceStore()
const activeId=ref(store.stickies[0]?.id ?? '')
const title=ref('')
const content=ref('')
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
async function remove(){const item=activeSticky.value;if(!item)return;if(!await saveNow())return;try{await api.deleteSticky(item.id);await store.reload();load(store.stickies[0]?.id??'');store.notify('便签已移入回收站')}catch(e){store.notify(String(e))}}
onMounted(()=>{unregisterSave=store.registerPendingSave(async()=>{if(!await saveNow())throw new Error('便签尚未保存')})})
onBeforeUnmount(()=>{if(saveTimer)clearTimeout(saveTimer);unregisterSave?.();if(activeId.value&&saveState.value==='未保存')void saveNow()})
watch(()=>store.stickies.length,(length)=>{if(!activeId.value&&length)load(store.stickies[0].id)})
if(activeSticky.value)load(activeSticky.value.id)
watch(()=>store.searchTarget,target=>{if(target?.kind==='sticky'){load(target.id);store.searchTarget=null}},{immediate:true})
</script>

<template>
  <div class="flex flex-col gap-8 pb-10">
    <div class="flex items-center justify-end gap-2" aria-label="页面操作"><Button @click="create"><Plus data-icon="inline-start" />新建便签</Button></div>
    <div class="grid min-h-[28rem] grid-cols-[16rem_minmax(0,1fr)] gap-5">
      <Card class="flex flex-col gap-3 p-3">
        <p class="px-3 py-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">我的便签 · {{ store.stickies.length }}</p>
        <div class="flex flex-col gap-1">
        <Button v-for="item in store.stickies" :key="item.id" variant="ghost" class="h-auto w-full flex-col items-stretch gap-1 rounded-xl px-3 py-3 text-left whitespace-normal" :class="activeId===item.id?'bg-accent':''" @click="select(item.id)"><span class="truncate text-sm font-medium">{{ item.title||'无标题便签' }}</span><span class="line-clamp-2 text-xs leading-5 text-muted-foreground">{{ item.content||'空白便签' }}</span></Button>
        </div>
        <p v-if="!store.stickies.length" class="px-3 py-8 text-center text-sm text-muted-foreground">创建便签开始记录</p>
      </Card>
      <Card v-if="activeSticky" class="flex flex-col gap-4 p-6">
        <div class="flex items-center gap-3"><StickyNote class="text-primary" /><Input v-model="title" aria-label="便签标题" class="min-w-0" placeholder="便签标题" /><ConfirmDialog title="删除便签？" :description="`确认删除“${activeSticky.title || '无标题便签'}”？内容将移入回收站，可在设置中恢复。`" confirm-text="删除便签" destructive @confirm="remove"><template #trigger><Button variant="ghost" size="icon" aria-label="删除便签"><Trash2 /></Button></template></ConfirmDialog></div>
        <Textarea v-model="content" aria-label="便签内容" class="min-h-[22rem] flex-1 resize-none" placeholder="把想法写在这里……" />
        <p class="flex items-center gap-2 text-xs text-muted-foreground"><Check class="size-3.5 text-primary" />{{ saveState||'自动保存已开启' }}</p>
      </Card>
      <Card v-else class="flex flex-col items-center justify-center p-10 text-center"><StickyNote class="mb-4 text-primary" /><h2 class="font-semibold">选择一条便签</h2><p class="mt-2 text-sm text-muted-foreground">也可以创建一条新的便签。</p></Card>
    </div>
  </div>
</template>
