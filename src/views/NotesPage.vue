<script setup lang="ts">
import MarkdownIt from 'markdown-it'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { BookOpenText, FilePlus2, Pencil, Search, Trash2, FileOutput, ChevronDown, Settings, Plus, Tags, Ellipsis } from '@lucide/vue'
import { save } from '@tauri-apps/plugin-dialog'
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { Textarea } from '@/components/ui/textarea'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuGroup, DropdownMenuCheckboxItem, DropdownMenuLabel, DropdownMenuItem } from '@/components/ui/dropdown-menu'
import { ContextMenu, ContextMenuTrigger, ContextMenuContent, ContextMenuGroup, ContextMenuItem } from '@/components/ui/context-menu'
import { ScrollArea } from '@/components/ui/scroll-area'
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
const recentNote=computed(()=>store.notes.reduce<(typeof store.notes)[number]|undefined>((latest,note)=>!latest||note.updated_at>latest.updated_at?note:latest,undefined))
const activeId=ref(recentNote.value?.id??'')
const title=ref('')
const content=ref('')
const filter=ref('')
const tagFilter=ref('all'), tagDraft=ref('')
const allTags=computed(()=>[...new Set(Object.values(store.settings.product.note_tags).flat())].sort())
const managingTags=ref(false)
const tagSearch=ref('')
const newTag=ref('')
const editingTag=ref('')
const renamedTag=ref('')
const tagBusy=ref(false)
const catalogKey='__tag_catalog__'
const managedTags=computed(()=>allTags.value.filter(tag=>tag.toLocaleLowerCase().includes(tagSearch.value.toLocaleLowerCase())))
function tagUsage(tag:string){return store.notes.filter(note=>store.settings.product.note_tags[note.id]?.includes(tag)).length}
async function openTagManager(){if(!await saveNow())return;managingTags.value=true}
async function changeTag(oldTag:string|null,newValue:string|null){
  const replacement=newValue?.trim()??null
  if(newValue!==null&&(!replacement||replacement.length>40||/[,，]/.test(replacement))){store.notify('标签需为 1–40 个字符，不能包含逗号');return}
  if(replacement&&replacement!==oldTag&&allTags.value.includes(replacement)){store.notify('标签已存在');return}
  if(!await saveNow())return
  tagBusy.value=true
  try{
    const tags:Record<string,string[]>={}
    for(const [id,values] of Object.entries(store.settings.product.note_tags))tags[id]=[...new Set(values.flatMap(tag=>tag===oldTag?(replacement?[replacement]:[]):[tag]))]
    const catalog=allTags.value.flatMap(tag=>tag===oldTag?(replacement?[replacement]:[]):[tag])
    if(oldTag===null&&replacement)catalog.push(replacement)
    // Keep unused labels in bounded catalog entries alongside the existing tag settings.
    for(const id of Object.keys(tags))if(id.startsWith(catalogKey))delete tags[id]
    for(let index=0;index<catalog.length;index+=20)tags[`${catalogKey}${index/20}`]=catalog.slice(index,index+20)
    await store.setSetting('product',{...store.settings.product,note_tags:tags})
    tagDraft.value=(tags[activeId.value]||[]).join('，')
    if(tagFilter.value===oldTag)tagFilter.value=replacement??'all'
    newTag.value='';editingTag.value='';store.notify('标签已保存')
  }catch(e){store.notify(String(e))}finally{tagBusy.value=false}
}
let tagSave:Promise<boolean>=Promise.resolve(true)
function saveTags():Promise<boolean>{const id=activeId.value;if(!id)return tagSave;const tags=[...new Set(tagDraft.value.split(/[,，]/).map(tag=>tag.trim()).filter(Boolean))].slice(0,20);if(JSON.stringify(tags)===JSON.stringify(store.settings.product.note_tags[id]||[]))return tagSave.then(()=>JSON.stringify(tags)===JSON.stringify(store.settings.product.note_tags[id]||[]));tagSave=tagSave.then(async()=>{try{await store.setSetting('product',{...store.settings.product,note_tags:{...store.settings.product.note_tags,[id]:tags}});return true}catch(e){store.notify(String(e));return false}});return tagSave}

const selectedTags=computed(()=>tagDraft.value.split(/[,，]/).map(tag=>tag.trim()).filter(Boolean))
const assigningTag=ref(false)
async function assignTag(tag:string,checked:boolean){
  if(assigningTag.value)return
  if(checked&&selectedTags.value.length>=20){store.notify('每篇笔记最多选择 20 个标签');return}
  const previous=tagDraft.value
  tagDraft.value=(checked?[...selectedTags.value,tag]:selectedTags.value.filter(value=>value!==tag)).join('，')
  assigningTag.value=true
  try{if(!await saveTags())tagDraft.value=previous}finally{assigningTag.value=false}
}
async function exportNote(id=activeId.value){if(!await saveNow())return;const note=store.notes.find(item=>item.id===id);if(!note)return;const filename=(note.title||'笔记').replace(/[<>:"/\\|?*]/g,'_');const path=await save({defaultPath:filename+'.md',filters:[{name:'Markdown',extensions:['md']}]});if(path)try{await api.exportMarkdown(id,path);store.notify('笔记已导出')}catch(e){store.notify(String(e))}}

const deleteNoteOpen=ref(false)
const deleteNoteId=ref('')
function askDeleteNote(id:string){deleteNoteId.value=id;deleteNoteOpen.value=true}
const mode=ref<'edit'|'preview'>('preview')
const saveState=ref('')
let saveTimer:ReturnType<typeof setTimeout>|undefined
let unregisterSave:(()=>void)|undefined
let saveChain:Promise<void>=Promise.resolve()
let loadingRecord=false
const current=computed(()=>store.notes.find(x=>x.id===activeId.value))
const filtered=computed(()=>store.notes.filter(n=>(tagFilter.value==='all'||store.settings.product.note_tags[n.id]?.includes(tagFilter.value))&&`${n.title} ${n.content}`.toLocaleLowerCase().includes(filter.value.toLocaleLowerCase())))
const rendered=computed(()=>md.render(content.value||'*这篇笔记还没有内容。*'))

function load(id:string){loadingRecord=true;const note=store.notes.find(x=>x.id===id);activeId.value=id;title.value=note?.title??'';content.value=note?.content??'';saveState.value='';tagDraft.value=(store.settings.product.note_tags[id]||[]).join('，');mode.value='preview';managingTags.value=false;loadingRecord=false}
async function saveNow():Promise<boolean>{
  if(!await saveTags())return false
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
async function select(id:string){if(id===activeId.value){managingTags.value=false;return;}if(!await saveNow())return;load(id)}
async function create(){if(!await saveNow())return;try{const item=await api.createNote('新笔记');await store.reload();load(item.id);mode.value='edit';store.notify('已新建笔记')}catch(e){store.notify(String(e))}}
async function remove(){const id=deleteNoteId.value;if(!id||!await saveNow())return;try{await api.deleteNote(id);await store.reload();if(id===activeId.value)load(recentNote.value?.id??'');store.notify('笔记已移入回收站')}catch(e){store.notify(String(e))}}
onMounted(()=>{unregisterSave=store.registerPendingSave(async()=>{if(!await saveNow())throw new Error('笔记尚未保存')})})
onBeforeUnmount(()=>{if(saveTimer)clearTimeout(saveTimer);unregisterSave?.();if(activeId.value&&saveState.value==='未保存')void saveNow()})
watch(()=>store.notes.length,length=>{if(!activeId.value&&length)load(recentNote.value?.id??'')})
if(current.value)load(current.value.id)
watch(()=>store.searchTarget,target=>{if(target?.kind==='note'){load(target.id);mode.value='preview';store.searchTarget=null}},{immediate:true})
async function openMarkdownLink(event:MouseEvent){const anchor=(event.target as HTMLElement).closest('a');if(!anchor?.href)return;event.preventDefault();try{await openUrl(anchor.href)}catch(e){store.notify(`无法打开链接：${String(e)}`)}}
</script>

<template>
  <div class="grid h-full min-h-0 flex-1 grid-cols-[17rem_minmax(0,1fr)] gap-5">

      <Card class="flex min-h-0 flex-col gap-3 overflow-hidden p-3">
        <div class="shrink-0 px-1"><p class="text-base font-semibold">我的笔记 · {{ store.notes.length }}</p><p class="mt-1 text-xs text-muted-foreground">适合长期整理：支持 Markdown、标签与导出。</p></div>
        <div v-if="store.noteFileIssues.length" role="alert" class="shrink-0 rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 text-xs leading-5 text-foreground">
          <p class="font-medium">有 {{ store.noteFileIssues.length }} 个 Markdown 文件未载入</p>
          <p class="mt-1 text-muted-foreground">原文件已保留。请检查文件头部格式；修复后重新打开笔记页即可载入。</p>
          <ScrollArea class="mt-2 max-h-24"><ul class="list-disc space-y-1 pl-4"><li v-for="issue in store.noteFileIssues" :key="issue" class="break-all">{{ issue }}</li></ul></ScrollArea>
        </div>
        <div class="flex shrink-0 items-center gap-2"><div class="relative min-w-0 flex-1"><Search class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" /><Input v-model="filter" class="pl-9" placeholder="搜索笔记" aria-label="搜索笔记" /></div><Button size="icon" aria-label="新建笔记" title="新建笔记" @click="create"><FilePlus2 /></Button></div>
        <div class="flex shrink-0 items-center gap-2"><Select v-model="tagFilter"><SelectTrigger class="min-w-0 flex-1" aria-label="按标签筛选笔记"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem value="all">全部标签</SelectItem><SelectItem v-for="tag in allTags" :key="tag" :value="tag">{{ tag }}</SelectItem></SelectGroup></SelectContent></Select><TooltipProvider><Tooltip><TooltipTrigger as-child><Button variant="ghost" size="icon" aria-label="标签管理" :aria-pressed="managingTags" @click="openTagManager"><Settings /></Button></TooltipTrigger><TooltipContent>标签管理</TooltipContent></Tooltip></TooltipProvider></div>
        <div class="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto overscroll-contain">
        <ContextMenu v-for="note in filtered" :key="note.id"><ContextMenuTrigger as-child><Button variant="ghost" class="h-auto w-full flex-col items-stretch gap-1 rounded-xl px-3 py-3 text-left whitespace-normal" :class="activeId===note.id?'bg-accent':''" @click="select(note.id)"><span class="truncate text-sm font-medium">{{ note.title||'无标题笔记' }}</span><span class="line-clamp-2 text-xs leading-5 text-muted-foreground">{{ note.content||'开始写下第一行……' }}</span></Button></ContextMenuTrigger><ContextMenuContent class="w-40"><ContextMenuGroup><ContextMenuItem @select="exportNote(note.id)"><FileOutput />导出</ContextMenuItem><ContextMenuItem variant="destructive" @select="askDeleteNote(note.id)"><Trash2 />删除</ContextMenuItem></ContextMenuGroup></ContextMenuContent></ContextMenu>
        </div>
        <p v-if="!filtered.length" class="px-3 py-8 text-center text-sm text-muted-foreground">{{ filter?'没有匹配的笔记':'创建第一篇笔记' }}</p>
      </Card>
            <Card v-if="managingTags" class="flex min-h-0 min-w-0 flex-col gap-4 overflow-hidden p-5">
        <div class="flex shrink-0 items-center justify-between"><h2 class="font-semibold">标签管理</h2><Button variant="ghost" @click="managingTags=false">返回笔记</Button></div>
        <Input v-model="tagSearch" aria-label="搜索标签" placeholder="搜索标签" />
        <form class="flex shrink-0 gap-2" @submit.prevent="changeTag(null,newTag)"><Input v-model="newTag" aria-label="新标签名称" placeholder="输入新标签名称" maxlength="40" /><Button type="submit" :disabled="tagBusy||!newTag.trim()"><Plus data-icon="inline-start" />新增标签</Button></form>
        <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto">
          <div v-for="tag in managedTags" :key="tag" class="flex shrink-0 items-center gap-3 rounded-xl border p-3">
            <template v-if="editingTag===tag"><Input v-model="renamedTag" aria-label="修改标签名称" maxlength="40" @keydown.enter="changeTag(tag,renamedTag)" /><Button :disabled="tagBusy" @click="changeTag(tag,renamedTag)">保存</Button><Button variant="ghost" @click="editingTag=''">取消</Button></template>
            <template v-else><div class="min-w-0 flex-1"><p class="truncate text-sm font-medium">{{ tag }}</p><p class="text-xs text-muted-foreground">{{ tagUsage(tag) }} 篇笔记</p></div><Button variant="ghost" size="icon" aria-label="重命名标签" :disabled="tagBusy" @click="editingTag=tag;renamedTag=tag"><Pencil /></Button><ConfirmDialog title="删除标签？" :description="`删除“${tag}”将移除所有笔记中的此标签，笔记内容会保留。`" destructive @confirm="changeTag(tag,null)"><template #trigger><Button variant="danger" size="icon" aria-label="删除标签" :disabled="tagBusy"><Trash2 /></Button></template></ConfirmDialog></template>
          </div>
          <p v-if="!managedTags.length" class="py-8 text-center text-sm text-muted-foreground">{{ tagSearch?'没有匹配的标签':'暂无标签，添加一个开始整理笔记。' }}</p>
        </div>
      </Card>
      <Tabs v-else-if="current" v-model="mode" class="min-h-0 min-w-0"><Card class="flex h-full min-h-0 min-w-0 flex-col gap-4 overflow-hidden p-5">
        <div class="flex shrink-0 items-center gap-3"><Input v-model="title" class="min-w-0" placeholder="无标题笔记" aria-label="笔记标题" /><div class="flex shrink-0 items-center gap-1"><TabsList aria-label="笔记显示模式"><TabsTrigger value="edit"><Pencil />编辑</TabsTrigger><TabsTrigger value="preview"><BookOpenText />预览</TabsTrigger></TabsList><DropdownMenu><DropdownMenuTrigger as-child><Button variant="outline" class="max-w-44" aria-label="选择笔记标签"><Tags data-icon="inline-start" /><span class="truncate">{{ selectedTags.length?`标签 · ${selectedTags.length}`:'标签' }}</span><ChevronDown data-icon="inline-end" /></Button></DropdownMenuTrigger><DropdownMenuContent align="end" class="max-h-72 w-56 overflow-y-auto"><DropdownMenuLabel>选择标签（可多选）</DropdownMenuLabel><DropdownMenuGroup><DropdownMenuCheckboxItem v-for="tag in allTags" :key="tag" :model-value="selectedTags.includes(tag)" :disabled="assigningTag" @select="$event.preventDefault()" @update:model-value="value=>assignTag(tag,value===true)">{{ tag }}</DropdownMenuCheckboxItem></DropdownMenuGroup><p v-if="!allTags.length" class="px-2 py-3 text-sm text-muted-foreground">请先在标签管理中新增标签。</p></DropdownMenuContent></DropdownMenu><DropdownMenu><DropdownMenuTrigger as-child><Button variant="outline" size="icon" aria-label="更多笔记操作"><Ellipsis /></Button></DropdownMenuTrigger><DropdownMenuContent align="end" class="w-44"><DropdownMenuGroup><DropdownMenuItem @select="exportNote()"><FileOutput />导出 Markdown</DropdownMenuItem><DropdownMenuItem variant="destructive" @select="askDeleteNote(activeId)"><Trash2 />删除</DropdownMenuItem></DropdownMenuGroup></DropdownMenuContent></DropdownMenu></div></div>
        <TabsContent value="edit" class="flex min-h-0 flex-1 overflow-hidden"><Textarea v-model="content" aria-label="笔记内容" class="h-full min-h-0 flex-1 resize-none overflow-y-auto" placeholder="# 新笔记\n\n在这里开始写作，支持常用 Markdown 语法。" /></TabsContent>
        <TabsContent value="preview" class="flex min-h-0 flex-1 overflow-hidden"><article class="markdown-preview min-h-0 max-w-none flex-1 overflow-auto rounded-2xl bg-muted/40 p-5 text-sm leading-7" v-html="rendered" @click="openMarkdownLink" /></TabsContent>
      </Card></Tabs>
      <Card v-else class="flex flex-col items-center justify-center p-10 text-center"><BookOpenText class="mb-4 text-primary" /><h2 class="font-semibold">{{ filter?'没有匹配的笔记':'选择一篇笔记' }}</h2><p class="mt-2 text-sm text-muted-foreground">使用常见 Markdown 语法记录与整理。</p></Card>
    <ConfirmDialog v-model:open="deleteNoteOpen" title="删除笔记？" description="内容将移入回收站，可在设置中恢复。" confirm-text="删除笔记" destructive @confirm="remove" />
  </div>
</template>
