<script setup lang="ts">
import { askConfirmation } from '@/lib/confirm'
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

import { computed, nextTick, reactive, ref, watch } from 'vue'
import { Monitor, Terminal, SquareTerminal, NotebookPen, Calculator, Settings, ChevronDown, LayoutGrid, AppWindow, ArrowUpRight, FileText, Folder, FolderPlus, Globe, Heart, Pencil, Plus, RefreshCw, Search, Star, Trash2 } from '@lucide/vue'
import { open } from '@tauri-apps/plugin-dialog'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Field } from '@/components/ui/field'
import { FieldGroup } from '@/components/ui/field'
import { FieldLabel } from '@/components/ui/field'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from '@/components/ui/dialog'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import { Checkbox } from '@/components/ui/checkbox'
import { ContextMenu, ContextMenuTrigger, ContextMenuContent, ContextMenuGroup, ContextMenuItem, ContextMenuSeparator, ContextMenuSub, ContextMenuSubTrigger, ContextMenuSubContent } from '@/components/ui/context-menu'
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem } from '@/components/ui/dropdown-menu'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { api } from '@/lib/api'
import { useWorkspaceStore } from '@/stores/workspace'
import type { Resource, ResourceKind, ScanCandidate } from '@/lib/types'

const store=useWorkspaceStore()
const commonDialogOpen=ref(false), commonSaving=ref(false), commonOpening=ref('')
const commonLaunchers=[
  {id:'computer',name:'此电脑',icon:Monitor,aliases:'我的电脑 this pc computer'},
  {id:'recycle_bin',name:'回收站',icon:Trash2,aliases:'recycle bin'},
  {id:'explorer',name:'文件资源管理器',icon:Folder,aliases:'explorer 资源管理器'},
  {id:'cmd',name:'CMD',icon:SquareTerminal,aliases:'命令提示符 command'},
  {id:'powershell',name:'PowerShell',icon:Terminal,aliases:'终端'},
  {id:'notepad',name:'记事本',icon:NotebookPen,aliases:'notepad'},
  {id:'calculator',name:'计算器',icon:Calculator,aliases:'calculator calc'},
]
const enabledCommon=computed(()=>commonLaunchers.filter(item=>(store.settings.common_launchers??commonLaunchers.map(x=>x.id)).includes(item.id)))
const shownCommon=computed(()=>activeFilter.value==='all'?enabledCommon.value.filter(item=>`${item.name} ${item.aliases}`.toLocaleLowerCase().includes(query.value.trim().toLocaleLowerCase())):[])
async function setCommon(id:string,enabled:boolean){if(commonSaving.value)return;commonSaving.value=true;try{const current=store.settings.common_launchers??commonLaunchers.map(x=>x.id);await store.setSetting('common_launchers',enabled?[...new Set([...current,id])]:current.filter(x=>x!==id))}catch{}finally{commonSaving.value=false}}
async function launchCommon(id:string){if(commonOpening.value)return;commonOpening.value=id;try{await api.launchCommon(id)}catch(e){store.notify(String(e))}finally{commonOpening.value=''}}
const activeFilter=ref('all')
const activeGroup=computed(()=>store.groups.find(group=>activeFilter.value===`group:${group.id}`))
const query=ref('')
const showForm=ref(false)
const editingId=ref('')
const groupName=ref('')
const groupDialogOpen=ref(false), groupEditingId=ref(''), groupSaving=ref(false)
const showScan=ref(false)
const candidates=ref<ScanCandidate[]>([])
const selectedTargets=ref<string[]>([])
const allScanSelected=computed(()=>candidates.value.length>0&&candidates.value.every(item=>selectedTargets.value.includes(item.target)))
function toggleAllScan(){selectedTargets.value=allScanSelected.value?[]:[...new Set(candidates.value.map(item=>item.target))]}
const scanBusy=ref(false)
const focusedId=ref('')
const openingId=ref('')
const form=reactive({name:'',kind:'app' as ResourceKind,target:'',group_id:'',aliases:''})
const filters=[{id:'all',label:'全部',icon:LayoutGrid},{id:'app',label:'应用',icon:AppWindow},{id:'web',label:'网页',icon:Globe},{id:'file',label:'文件',icon:FileText}] as const
watch(()=>store.searchTarget,async target=>{if(target?.kind==='resource'){focusedId.value=target.id;activeFilter.value='all';await nextTick();document.getElementById(`resource-${target.id}`)?.scrollIntoView({behavior:'smooth',block:'center'});store.searchTarget=null}},{immediate:true})
const matchingIds=ref<string[]>([])
let searchVersion=0
watch([query,()=>store.resources],async ([value])=>{const version=++searchVersion;matchingIds.value=[];try{const results=await api.searchResources(value);if(version===searchVersion)matchingIds.value=results.map(item=>item.id)}catch(e){store.notify(String(e))}})
const shownResources=computed(()=>{
  const list=store.resources.filter(r=>(activeFilter.value==='all'||activeFilter.value===r.kind||(activeFilter.value==='favorites'&&r.is_favorite)||activeFilter.value===`group:${r.group_id}`)&&(!query.value.trim()||matchingIds.value.includes(r.id)))
  return list.sort((a,b)=>query.value.trim()?matchingIds.value.indexOf(a.id)-matchingIds.value.indexOf(b.id):Number(b.is_favorite)-Number(a.is_favorite)||b.usage_count-a.usage_count||(b.last_opened_at??'').localeCompare(a.last_opened_at??'')||a.name.localeCompare(b.name))
})
const batchMode=ref(false), selectedIds=ref<string[]>([]), batchBusy=ref(false), deleteBatch=ref(false)
function toggleSelection(id:string){selectedIds.value=selectedIds.value.includes(id)?selectedIds.value.filter(x=>x!==id):[...selectedIds.value,id]}
async function batch(action:'move'|'favorite'|'unfavorite'|'delete',groupId:string|null=null,ids=selectedIds.value){
  batchBusy.value=true
  try{await api.batchResources(ids,action,groupId);await store.reload();selectedIds.value=[];store.notify('操作完成')}catch(e){store.notify(String(e))}finally{batchBusy.value=false}
}
async function reveal(item:Resource){try{await api.revealResource(item.id)}catch(e){store.notify(String(e))}}
async function copyPath(item:Resource){try{await navigator.clipboard.writeText(item.target);store.notify('路径已复制')}catch(e){store.notify(`复制失败：${String(e)}`)}}

function clearForm(){showForm.value=false;editingId.value='';form.name='';form.kind='app';form.target='';form.group_id='';form.aliases=''}
function addResource(){clearForm();showForm.value=true}
function edit(item:Resource){editingId.value=item.id;form.name=item.name;form.kind=item.kind;form.target=item.target;form.group_id=item.group_id??'';form.aliases=item.aliases;showForm.value=true}
async function pickTarget(){const result=await open({multiple:false,directory:false,filters:form.kind==='app'?[{name:'应用程序或快捷方式',extensions:['exe','lnk']}]:undefined});if(typeof result==='string'){form.target=result;if(/\.(exe|lnk)$/i.test(result))form.kind='app'}}
async function save(){if(!form.name.trim()||!form.target.trim()){store.notify('请填写名称和启动目标');return}try{if(editingId.value){const old=store.resources.find(x=>x.id===editingId.value);if(old)await api.updateResource({...old,name:form.name.trim(),kind:form.kind,target:form.target.trim(),group_id:form.group_id||null,aliases:form.aliases.trim()})}else await api.createResource({name:form.name.trim(),kind:form.kind,target:form.target.trim(),group_id:form.group_id||null,aliases:form.aliases.trim()});await store.reload();store.notify('启动项已保存');clearForm()}catch(e){store.notify(`保存失败：${String(e)}`)}}
async function remove(item:Resource){if(!await askConfirmation(`从应用库移除“${item.name}”？原始程序或文件不会删除。`))return;try{await api.deleteResource(item.id);await store.reload();store.notify('已从应用库移除')}catch(e){store.notify(String(e))}}
async function launch(item:Resource){if(openingId.value)return;openingId.value=item.id;try{await store.launchResource(item.id)}catch(e){store.notify(String(e));if(item.kind!=='web'&&await askConfirmation('该路径可能已失效。现在重新定位吗？'))await relocate(item)}finally{openingId.value=''}}
async function relocate(item:Resource){const result=await open({multiple:false,directory:false,filters:item.kind==='app'?[{name:'应用程序或快捷方式',extensions:['exe','lnk']}]:undefined});if(typeof result!=='string')return;try{await api.relocateResource(item.id,result);await store.reload();store.notify('路径已更新')}catch(e){store.notify(String(e))}}
async function favorite(item:Resource){try{await api.toggleFavorite(item.id);await store.reload()}catch(e){store.notify(String(e))}}
function promptNewGroup(){groupEditingId.value='';groupName.value='';groupDialogOpen.value=true}
function renameGroup(id:string,name:string){groupEditingId.value=id;groupName.value=name;groupDialogOpen.value=true}
async function saveGroup(){if(!groupName.value.trim()||groupSaving.value)return;groupSaving.value=true;try{if(groupEditingId.value)await api.renameGroup(groupEditingId.value,groupName.value.trim());else await api.createGroup(groupName.value.trim());await store.reload();groupDialogOpen.value=false;store.notify(groupEditingId.value?'分组名称已更新':'分组已创建')}catch(e){store.notify(String(e))}finally{groupSaving.value=false}}
async function removeGroup(id:string,name:string){if(!await askConfirmation(`删除分组“${name}”？其中的启动项会移至未分组。`))return;try{await api.deleteGroup(id);if(activeFilter.value===`group:${id}`)activeFilter.value='all';await store.reload();store.notify('分组已删除')}catch(e){store.notify(String(e))}}
async function scan(){scanBusy.value=true;showScan.value=true;try{candidates.value=await api.scanApplications();selectedTargets.value=[]}catch(e){store.notify(`扫描失败：${String(e)}`)}finally{scanBusy.value=false}}
async function importSelected(){const selected=candidates.value.filter(c=>selectedTargets.value.includes(c.target));if(!selected.length){store.notify('请先选择要导入的应用');return}try{const count=await api.importCandidates(selected);await store.reload();showScan.value=false;store.notify(`已导入或更新 ${count} 个应用`)}catch(e){store.notify(`导入失败：${String(e)}`)}}
function iconFor(kind:ResourceKind){return kind==='app'?AppWindow:kind==='web'?Globe:FileText}
function scrollFilters(event:WheelEvent){const element=event.currentTarget as HTMLElement;if(event.shiftKey||Math.abs(event.deltaX)>Math.abs(event.deltaY)||element.scrollWidth<=element.clientWidth)return;const before=element.scrollLeft;element.scrollLeft+=event.deltaY;if(element.scrollLeft!==before)event.preventDefault()}
</script>

<template>
  <div class="flex flex-col gap-8 pb-10">
    <Dialog v-model:open="commonDialogOpen"><DialogContent class="flex max-h-[85dvh] flex-col"><DialogHeader><DialogTitle>配置常用启动项</DialogTitle><DialogDescription>开启的项目显示在「全部」的第一行，修改后自动保存。</DialogDescription></DialogHeader><FieldGroup class="min-h-0 gap-3 overflow-y-auto p-1"><Field v-for="item in commonLaunchers" :key="item.id" orientation="horizontal"><FieldLabel :for="`common-${item.id}`"><component :is="item.icon" class="size-5 shrink-0" />{{ item.name }}</FieldLabel><Switch :id="`common-${item.id}`" :model-value="enabledCommon.some(x=>x.id===item.id)" :disabled="commonSaving" @update:model-value="value=>setCommon(item.id,value)" /></Field></FieldGroup><DialogFooter><Button @click="commonDialogOpen=false">完成</Button></DialogFooter></DialogContent></Dialog>
    <Dialog v-model:open="groupDialogOpen"><DialogContent><DialogHeader><DialogTitle>{{ groupEditingId?'重命名分组':'新建分组' }}</DialogTitle><DialogDescription>输入分组名称，整理应用库中的启动项。</DialogDescription></DialogHeader><form class="flex flex-col gap-5" @submit.prevent="saveGroup"><FieldGroup><Field><FieldLabel for="group-name">分组名称</FieldLabel><Input id="group-name" v-model="groupName" placeholder="输入分组名称" autofocus :maxlength="80" /></Field></FieldGroup><DialogFooter><Button type="button" variant="outline" @click="groupDialogOpen=false">取消</Button><Button type="submit" :disabled="!groupName.trim()||groupSaving">{{ groupSaving?'正在保存…':'保存' }}</Button></DialogFooter></form></DialogContent></Dialog>
    <Dialog :open="showForm" @update:open="value=>{if(!value)clearForm()}"><DialogContent class="flex max-h-[85dvh] flex-col sm:max-w-xl">
      <DialogHeader><DialogTitle>{{ editingId?'编辑启动项':'添加启动项' }}</DialogTitle><DialogDescription>填写启动项信息，将应用、网页或文件添加到应用库。</DialogDescription></DialogHeader>
      <form class="flex min-h-0 flex-col gap-5" @submit.prevent="save">
      <FieldGroup class="min-h-0 overflow-y-auto p-1">
        <Field><FieldLabel for="resource-name">名称</FieldLabel><Input id="resource-name" v-model="form.name" placeholder="例如：浏览器、项目文件" /></Field>
        <Field><FieldLabel for="resource-kind">类型</FieldLabel><Select v-model="form.kind"><SelectTrigger id="resource-kind" class="w-full"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem value="app">应用</SelectItem><SelectItem value="web">网页</SelectItem><SelectItem value="file">文件</SelectItem></SelectGroup></SelectContent></Select></Field>
        <Field><FieldLabel for="resource-target">{{ form.kind==='web'?'网址':'程序或文件路径' }}</FieldLabel><div class="flex gap-2"><Input id="resource-target" v-model="form.target" :placeholder="form.kind==='web'?'https://example.com':'选择或输入本地路径'" /><Button v-if="form.kind!=='web'" type="button" variant="outline" @click="pickTarget">浏览</Button></div></Field>
        <Field><FieldLabel for="resource-aliases">搜索别名</FieldLabel><Input id="resource-aliases" v-model="form.aliases" placeholder="例如：浏览器 chrome 谷歌" /></Field>
        <Field><FieldLabel for="resource-group">分组</FieldLabel><Select :model-value="form.group_id || 'ungrouped'" @update:model-value="value=>form.group_id=String(value)==='ungrouped'?'':String(value)"><SelectTrigger id="resource-group" class="w-full"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem value="ungrouped">未分组</SelectItem><SelectItem v-for="group in store.groups" :key="group.id" :value="group.id">{{ group.name }}</SelectItem></SelectGroup></SelectContent></Select></Field>
      </FieldGroup>
      <DialogFooter class="shrink-0"><Button type="button" variant="outline" @click="clearForm">取消</Button><Button type="submit">保存启动项</Button></DialogFooter>
      </form>
    </DialogContent></Dialog>
    <Dialog v-model:open="showScan"><DialogContent class="flex max-h-[85dvh] flex-col sm:max-w-3xl">
      <DialogHeader><DialogTitle>扫描本机应用</DialogTitle><DialogDescription>扫描常见开始菜单和已安装应用入口；只会导入你勾选的项目。</DialogDescription></DialogHeader>
      <p v-if="!scanBusy" role="status" class="text-sm text-muted-foreground">共 {{ candidates.length }} 个结果 · 已选择 {{ selectedTargets.length }} 个</p>
      <p v-if="scanBusy" class="py-6 text-center text-sm text-muted-foreground">正在扫描……</p>
      <div v-else-if="candidates.length" class="flex min-h-0 max-h-[55dvh] flex-col gap-2 overflow-y-auto overscroll-contain rounded-2xl border border-border p-3">
        <label v-for="item in candidates" :key="item.target" class="flex min-w-0 items-center gap-3 rounded-xl px-3 py-3 hover:bg-accent">
          <Checkbox :model-value="selectedTargets.includes(item.target)" :aria-label="`选择 ${item.name}`" @update:model-value="value=>selectedTargets=value===true?[...selectedTargets,item.target]:selectedTargets.filter(target=>target!==item.target)" />
          <img v-if="item.icon_data" :src="item.icon_data" alt="" class="size-8 shrink-0 object-contain" /><AppWindow v-else class="size-8 shrink-0 text-muted-foreground" />
          <span class="flex min-w-0 flex-1 flex-col gap-1">
            <span class="flex items-center gap-3"><span class="truncate text-sm font-medium">{{ item.name }}</span><span class="shrink-0 text-xs text-muted-foreground">{{ item.source }}</span></span>
            <span class="truncate text-xs text-muted-foreground" :title="item.installation_path??item.target">安装位置：{{ item.installation_path??'未能识别' }}</span>
            <span class="truncate text-xs text-muted-foreground" :title="item.target">启动入口：{{ item.target }}</span>
          </span>
        </label>
      </div>
      <p v-else-if="!scanBusy" class="py-6 text-center text-sm text-muted-foreground">没有找到可导入的应用。</p>
      <DialogFooter class="shrink-0"><Button variant="outline" :disabled="scanBusy||!candidates.length" @click="toggleAllScan">{{ allScanSelected?'取消全选':'全选' }}</Button><Button variant="outline" @click="showScan=false">取消</Button><Button :disabled="scanBusy||!selectedTargets.length" @click="importSelected">导入所选（{{ selectedTargets.length }}）</Button></DialogFooter>
    </DialogContent></Dialog>
    <Tabs v-model="activeFilter" class="flex min-w-0 flex-col gap-4">
      <div class="flex min-w-0 flex-wrap items-center gap-3">
        <div class="flex min-w-0 flex-1 basis-80 items-center gap-2 overflow-x-auto py-1" aria-label="应用库筛选和分组" @wheel="scrollFilters">
          <TabsList class="shrink-0" aria-label="应用库筛选">
            <TabsTrigger v-for="item in filters" :key="item.id" :value="item.id"><component :is="item.icon" />{{ item.label }}</TabsTrigger>
            <TabsTrigger value="favorites"><Star />收藏</TabsTrigger>
            <TabsTrigger v-for="group in store.groups" :key="group.id" :value="`group:${group.id}`"><Folder />{{ group.name }}</TabsTrigger>
          </TabsList>
          <Button variant="ghost" size="icon-xs" aria-label="配置常用启动项" @click="commonDialogOpen=true"><Settings /></Button>
          <template v-if="activeGroup">
            <Button variant="ghost" size="icon-xs" :aria-label="`重命名分组 ${activeGroup.name}`" @click="renameGroup(activeGroup.id,activeGroup.name)"><Pencil /></Button>
            <Button variant="ghost" size="icon-xs" :aria-label="`删除分组 ${activeGroup.name}`" @click="removeGroup(activeGroup.id,activeGroup.name)"><Trash2 /></Button>
          </template>
          <Button variant="ghost" size="sm" class="shrink-0" @click="promptNewGroup"><FolderPlus data-icon="inline-start" />新建分组</Button>
        </div>
        <div class="ml-auto flex min-w-0 flex-wrap items-center justify-end gap-2" aria-label="应用库操作">
          <Button variant="outline" size="sm" @click="batchMode=!batchMode;selectedIds=[]">{{ batchMode?'完成管理':'管理应用' }}</Button>
          <div class="relative w-48 shrink-0"><Search class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" /><Input v-model="query" class="h-8 pl-9" placeholder="名称、拼音或别名" aria-label="筛选启动项" /></div>
          <DropdownMenu><DropdownMenuTrigger as-child><Button size="sm"><Plus data-icon="inline-start" />添加<ChevronDown data-icon="inline-end" /></Button></DropdownMenuTrigger><DropdownMenuContent align="end"><DropdownMenuGroup><DropdownMenuItem @select="addResource"><Plus />添加启动项</DropdownMenuItem><DropdownMenuItem :disabled="scanBusy" @select="scan"><RefreshCw />扫描本机应用</DropdownMenuItem><DropdownMenuItem @select="commonDialogOpen=true"><Settings />配置常用启动项</DropdownMenuItem></DropdownMenuGroup></DropdownMenuContent></DropdownMenu>
        </div>
      </div>
      <TabsContent :value="activeFilter" class="flex flex-col gap-4">
        <div v-if="shownCommon.length" class="flex min-w-0 gap-4 overflow-x-auto" aria-label="常用启动项"><Button v-for="item in shownCommon" :key="item.id" variant="ghost" class="h-28 w-28 shrink-0 flex-col gap-3 rounded-2xl" :disabled="!!commonOpening" @click="launchCommon(item.id)"><component :is="item.icon" class="size-10 text-primary" /><span class="w-full truncate text-center text-sm">{{ item.name }}</span></Button></div>
        <div v-if="batchMode" class="flex flex-wrap items-center gap-3 rounded-xl bg-muted p-3">
          <Checkbox :model-value="shownResources.length>0&&shownResources.every(item=>selectedIds.includes(item.id))" aria-label="选择当前结果" @update:model-value="value=>selectedIds=value===true?shownResources.map(item=>item.id):[]"/><span class="text-sm">已选 {{ selectedIds.length }} 项</span>
          <DropdownMenu><DropdownMenuTrigger as-child><Button variant="outline" size="sm" :disabled="!selectedIds.length||batchBusy">移动分组</Button></DropdownMenuTrigger><DropdownMenuContent><DropdownMenuGroup><DropdownMenuItem @select="batch('move',null)">未分组</DropdownMenuItem><DropdownMenuItem v-for="group in store.groups" :key="group.id" @select="batch('move',group.id)">{{ group.name }}</DropdownMenuItem></DropdownMenuGroup></DropdownMenuContent></DropdownMenu>
          <Button variant="outline" size="sm" :disabled="!selectedIds.length||batchBusy" @click="batch('favorite')">收藏</Button><Button variant="outline" size="sm" :disabled="!selectedIds.length||batchBusy" @click="batch('unfavorite')">取消收藏</Button><Button variant="destructive" size="sm" :disabled="!selectedIds.length||batchBusy" @click="deleteBatch=true">移除所选</Button>
        </div>
        <div v-if="shownResources.length" :class="batchMode?'grid grid-cols-[repeat(auto-fill,minmax(min(100%,22rem),1fr))] gap-4':'grid grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-4'">
          <ContextMenu v-for="item in shownResources" :key="item.id"><ContextMenuTrigger as-child><Button v-if="!batchMode" :id="`resource-${item.id}`" variant="ghost" class="h-28 min-w-0 flex-col gap-3 rounded-2xl" :class="focusedId===item.id?'ring-2 ring-primary':''" :title="item.name" :disabled="!!openingId" @click="launch(item)"><img v-if="item.icon_data" :src="item.icon_data" alt="" class="size-10 object-contain"/><component :is="iconFor(item.kind)" v-else class="size-10 text-primary"/><span class="w-full truncate text-center text-sm">{{ item.name }}</span></Button><Card v-else :id="`resource-${item.id}`" :key="item.id" class="flex min-w-0 flex-col gap-4 p-5 transition-shadow" :class="focusedId===item.id?'ring-2 ring-primary':''">
            <div class="flex min-w-0 items-start gap-3"><Checkbox v-if="batchMode" :model-value="selectedIds.includes(item.id)" :aria-label="`选择 ${item.name}`" @update:model-value="toggleSelection(item.id)"/><div class="flex size-11 shrink-0 items-center justify-center rounded-2xl bg-primary/10 text-primary"><img v-if="item.icon_data" :src="item.icon_data" alt="" class="size-8 object-contain" /><component :is="iconFor(item.kind)" v-else /></div><div class="min-w-0 flex-1"><h2 class="truncate font-semibold">{{ item.name }}</h2><p class="mt-1 truncate text-xs text-muted-foreground">{{ item.target }}</p></div><Button variant="ghost" size="icon" :aria-label="item.is_favorite?'取消收藏':'收藏'" @click="favorite(item)"><Heart :class="item.is_favorite?'fill-primary text-primary':'text-muted-foreground'" /></Button></div>
            <div class="flex flex-wrap items-center justify-between gap-2"><Badge>{{ item.kind==='app'?'应用':item.kind==='web'?'网页':'文件' }}</Badge><span class="flex gap-1"><Button v-if="item.kind!=='web'" variant="ghost" size="sm" @click="relocate(item)">重新定位</Button><Button variant="ghost" size="icon" aria-label="编辑" @click="edit(item)"><Pencil /></Button><Button variant="ghost" size="icon" aria-label="删除" @click="remove(item)"><Trash2 /></Button><Button variant="outline" size="sm" @click="launch(item)">打开<ArrowUpRight data-icon="inline-end" /></Button></span></div>
          </Card></ContextMenuTrigger>
          <ContextMenuContent class="w-52"><ContextMenuGroup>
            <ContextMenuItem @select="launch(item)">打开</ContextMenuItem><ContextMenuItem :disabled="item.kind==='web'" @select="reveal(item)">打开所在文件夹</ContextMenuItem><ContextMenuItem @select="copyPath(item)">复制{{ item.kind==='web'?'网址':'路径' }}</ContextMenuItem><ContextMenuItem @select="favorite(item)">{{ item.is_favorite?'取消收藏':'收藏' }}</ContextMenuItem>
            <ContextMenuSub><ContextMenuSubTrigger>移动分组</ContextMenuSubTrigger><ContextMenuSubContent><ContextMenuGroup><ContextMenuItem @select="batch('move',null,[item.id])">未分组</ContextMenuItem><ContextMenuItem v-for="group in store.groups" :key="group.id" @select="batch('move',group.id,[item.id])">{{ group.name }}</ContextMenuItem></ContextMenuGroup></ContextMenuSubContent></ContextMenuSub>
            <ContextMenuItem @select="edit(item)">编辑名称、别名和路径</ContextMenuItem><ContextMenuItem @select="batchMode=true;toggleSelection(item.id)">选择 / 取消选择</ContextMenuItem>
          </ContextMenuGroup><ContextMenuSeparator/><ContextMenuGroup><ContextMenuItem variant="destructive" @select="remove(item)">从应用库移除</ContextMenuItem></ContextMenuGroup></ContextMenuContent></ContextMenu>
        </div>
        <Card v-else-if="!shownCommon.length" class="flex min-h-64 flex-col items-center justify-center p-8 text-center"><AppWindow class="mb-4 text-primary" /><h2 class="font-semibold">{{ store.resources.length?'没有匹配的启动项':'还没有启动项' }}</h2><p class="mt-2 max-w-sm text-sm text-muted-foreground">{{ store.resources.length?'试试切换筛选条件，或调整搜索关键词。':'手动添加应用、网页或文件，也可以扫描常见已安装应用并选择导入。' }}</p><Button class="mt-5" variant="outline" @click="addResource"><Plus data-icon="inline-start" />添加启动项</Button></Card>
      </TabsContent>
    </Tabs>
  </div>
  <ConfirmDialog v-model:open="deleteBatch" title="移除所选启动项？" description="只移除应用库中的记录，不会删除电脑中的原始程序或文件。" confirm-text="移除" destructive @confirm="batch('delete')" />
</template>

