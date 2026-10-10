<script setup lang="ts">
import MarkdownIt from 'markdown-it'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { BookOpenText, Check, FilePlus2, Pencil, Search, Trash2 } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { Textarea } from '@/components/ui/textarea'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { api } from '@/lib/api'
import { useWorkspaceStore } from '@/stores/workspace'

const store=useWorkspaceStore()
const md=new MarkdownIt({html:false,linkify:true,breaks:true})
const baseLinkOpen=md.renderer.rules.link_open
md.renderer.rules.link_open=(tokens,index,options,env,self)=>{
  tokens[index].attrSet('target','_blank')
  tokens[index].attrSet('rel','noopener noreferrer')
  return baseLinkOpen?baseLinkOpen(tokens,index,options,env,self):self.renderToken(tokens,index,options)
}
const activeId=ref(store.notes[0]?.id??'')
const title=ref('')
const content=ref('')
const filter=ref('')
const mode=ref<'edit'|'preview'>('edit')
const saveState=ref('')
let saveTimer:ReturnType<typeof setTimeout>|undefined
let unregisterSave:(()=>void)|undefined
let saveChain:Promise<void>=Promise.resolve()
let loadingRecord=false
const current=computed(()=>store.notes.find(x=>x.id===activeId.value))
const filtered=computed(()=>store.notes.filter(n=>`${n.title} ${n.content}`.toLocaleLowerCase().includes(filter.value.toLocaleLowerCase())))
const rendered=computed(()=>md.render(content.value||'*这篇笔记还没有内容。*'))

function load(id:string){loadingRecord=true;const note=store.notes.find(x=>x.id===id);activeId.value=id;title.value=note?.title??'';content.value=note?.content??'';saveState.value='';mode.value='edit';loadingRecord=false}
async function saveNow():Promise<boolean>{
  if(saveTimer)clearTimeout(saveTimer)
  const id=activeId.value
  if(!id)return true
  if(saveState.value===''||saveState.value==='已保存'){await saveChain;return true}
  while(activeId.value===id){
    const savedTitle=title.value
    const savedContent=content.value
    saveState.value='保存中…'
    const job=saveChain.then(()=>api.updateNote(id,savedTitle,savedContent))
    saveChain=job.catch(()=>{})
    try{
      await job
      const note=store.notes.find(x=>x.id===id)
      if(note){note.title=savedTitle;note.content=savedContent;note.updated_at=new Date().toISOString()}
      if(activeId.value!==id)return true
      if(title.value===savedTitle&&content.value===savedContent){saveState.value='已保存';return true}
    }catch(e){
      if(activeId.value===id)saveState.value='保存失败'
      store.notify(`笔记保存失败：${String(e)}`)
      return false
    }
  }
  return true
}
watch([title,content],()=>{if(loadingRecord||!activeId.value)return;saveState.value='未保存';if(saveTimer)clearTimeout(saveTimer);saveTimer=setTimeout(()=>void saveNow(),600)},{flush:'sync'})
async function select(id:string){if(id===activeId.value)return;if(!await saveNow())return;load(id)}
async function create(){if(!await saveNow())return;try{const item=await api.createNote('新笔记');await store.reload();load(item.id);store.notify('已新建笔记')}catch(e){store.notify(String(e))}}
async function remove(){if(!current.value||!await saveNow())return;try{await api.deleteNote(activeId.value);await store.reload();load(store.notes[0]?.id??'');store.notify('笔记已移入回收站')}catch(e){store.notify(String(e))}}
onMounted(()=>{unregisterSave=store.registerPendingSave(async()=>{if(!await saveNow())throw new Error('笔记尚未保存')})})
onBeforeUnmount(()=>{if(saveTimer)clearTimeout(saveTimer);unregisterSave?.();if(activeId.value&&saveState.value==='未保存')void saveNow()})
watch(()=>store.notes.length,length=>{if(!activeId.value&&length)load(store.notes[0].id)})
if(current.value)load(current.value.id)
watch(()=>store.searchTarget,target=>{if(target?.kind==='note'){load(target.id);mode.value='preview';store.searchTarget=null}},{immediate:true})
async function openMarkdownLink(event:MouseEvent){const anchor=(event.target as HTMLElement).closest('a');if(!anchor?.href)return;event.preventDefault();try{await openUrl(anchor.href)}catch(e){store.notify(`无法打开链接：${String(e)}`)}}
</script>

<template>
  <div class="flex flex-col gap-8 pb-10">
    <div class="flex items-center justify-end gap-2" aria-label="页面操作"><Button @click="create"><FilePlus2 data-icon="inline-start" />新建笔记</Button></div>
    <div class="grid min-h-[34rem] grid-cols-[17rem_minmax(0,1fr)] gap-5">
      <Card class="flex min-h-0 flex-col gap-3 p-3">
        <div class="relative"><Search class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" /><Input v-model="filter" class="pl-9" placeholder="搜索笔记" aria-label="搜索笔记" /></div>
        <div class="flex flex-col gap-1">
        <Button v-for="note in filtered" :key="note.id" variant="ghost" class="h-auto w-full flex-col items-stretch gap-1 rounded-xl px-3 py-3 text-left whitespace-normal" :class="activeId===note.id?'bg-accent':''" @click="select(note.id)"><span class="truncate text-sm font-medium">{{ note.title||'无标题笔记' }}</span><span class="line-clamp-2 text-xs leading-5 text-muted-foreground">{{ note.content||'开始写下第一行……' }}</span></Button>
        </div>
        <p v-if="!filtered.length" class="px-3 py-8 text-center text-sm text-muted-foreground">{{ filter?'没有匹配的笔记':'创建第一篇笔记' }}</p>
      </Card>
      <Tabs v-if="current" v-model="mode" class="min-w-0"><Card class="flex h-full min-w-0 flex-col gap-4 p-5">
        <div class="flex items-center gap-3"><Input v-model="title" class="min-w-0" placeholder="无标题笔记" aria-label="笔记标题" /><div class="flex shrink-0 items-center gap-1"><TabsList aria-label="笔记显示模式"><TabsTrigger value="edit"><Pencil />编辑</TabsTrigger><TabsTrigger value="preview"><BookOpenText />预览</TabsTrigger></TabsList><ConfirmDialog title="删除笔记？" description="内容将移入回收站，可在设置中恢复。" confirm-text="删除笔记" destructive @confirm="remove"><template #trigger><Button variant="ghost" size="icon" aria-label="删除笔记"><Trash2 /></Button></template></ConfirmDialog></div></div>
        <TabsContent value="edit" class="flex flex-1"><Textarea v-model="content" aria-label="笔记内容" class="min-h-[26rem] flex-1 resize-none" placeholder="# 新笔记\n\n在这里开始写作，支持常用 Markdown 语法。" /></TabsContent>
        <TabsContent value="preview" class="flex flex-1"><article class="markdown-preview min-h-[26rem] max-w-none flex-1 overflow-auto rounded-2xl bg-muted/40 p-5 text-sm leading-7" v-html="rendered" @click="openMarkdownLink" /></TabsContent>
        <p class="flex items-center gap-2 text-xs text-muted-foreground"><Check class="size-3.5 text-primary" />{{ saveState||'自动保存已开启' }}</p>
      </Card></Tabs>
      <Card v-else class="flex flex-col items-center justify-center p-10 text-center"><BookOpenText class="mb-4 text-primary" /><h2 class="font-semibold">{{ filter?'没有匹配的笔记':'选择一篇笔记' }}</h2><p class="mt-2 text-sm text-muted-foreground">使用常见 Markdown 语法记录与整理。</p></Card>
    </div>
  </div>
</template>
