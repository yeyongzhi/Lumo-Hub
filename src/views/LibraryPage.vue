<script setup lang="ts">
import { fixedResourceIds } from '@/lib/launcher-preferences'
import LibraryManagement from '@/components/LibraryManagement.vue'
import GroupManagement from '@/components/GroupManagement.vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { resourceImage, launchOptions, targetKey, stableResourceOrder } from '@/lib/library-preferences'
import { writeLocal } from '@/lib/local-files'
import { askConfirmation } from '@/lib/confirm'
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

import { computed, onUnmounted, onMounted, nextTick, reactive, ref, watch } from 'vue'
import { GripVertical, Monitor, Terminal, SquareTerminal, NotebookPen, Calculator, Settings, Settings2, Cpu, Ellipsis, LayoutGrid, AppWindow, ArrowUpRight, FileText, Folder, FolderCog, Globe, Heart, Pencil, Plus, RefreshCw, Search, Star, Trash2 } from '@lucide/vue'
import { open } from '@tauri-apps/plugin-dialog'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Field } from '@/components/ui/field'
import { FieldGroup } from '@/components/ui/field'
import { FieldLabel } from '@/components/ui/field'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from '@/components/ui/dialog'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { Textarea } from '@/components/ui/textarea'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import { Checkbox } from '@/components/ui/checkbox'
import { ContextMenu, ContextMenuTrigger, ContextMenuContent, ContextMenuGroup, ContextMenuItem, ContextMenuSeparator, ContextMenuSub, ContextMenuSubTrigger, ContextMenuSubContent } from '@/components/ui/context-menu'
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuCheckboxItem } from '@/components/ui/dropdown-menu'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { api } from '@/lib/api'
import { useWorkspaceStore } from '@/stores/workspace'
import type { Resource, ResourceKind, ScanCandidate } from '@/lib/types'

const store=useWorkspaceStore()
const commonDialogOpen=ref(false), commonSaving=ref(false), commonOpening=reactive(new Set<string>())
const commonIcons=ref<Record<string,string>>({})
onMounted(async()=>{try{commonIcons.value=await api.commonLauncherIcons()}catch(e){store.notify(`系统图标加载失败：${String(e)}`)}})
const commonLaunchers=[
  {id:'computer',name:'此电脑',icon:Monitor,aliases:'我的电脑 this pc computer'},
  {id:'recycle_bin',name:'回收站',icon:Trash2,aliases:'recycle bin'},
  {id:'explorer',name:'文件资源管理器',icon:Folder,aliases:'explorer 资源管理器'},
  {id:'cmd',name:'CMD',icon:SquareTerminal,aliases:'命令提示符 command'},
  {id:'powershell',name:'PowerShell',icon:Terminal,aliases:'终端'},
  {id:'control_panel',name:'控制面板',icon:Settings,aliases:'control panel 系统设置'},
  {id:'notepad',name:'记事本',icon:NotebookPen,aliases:'notepad'},
  {id:'calculator',name:'计算器',icon:Calculator,aliases:'calculator calc'},
  {id:'task_manager',name:'任务管理器',icon:Cpu,aliases:'task manager taskmgr 进程'},
  {id:'windows_settings',name:'设置',icon:Settings2,aliases:'windows settings ms-settings 系统设置'},
]
const enabledCommon=computed(()=>commonLaunchers.filter(item=>(store.settings.common_launchers??commonLaunchers.map(x=>x.id)).includes(item.id)))
const shownCommon=computed(()=>['all','common'].includes(activeFilter.value)?enabledCommon.value.filter(item=>`${item.name} ${item.aliases}`.toLocaleLowerCase().includes(query.value.trim().toLocaleLowerCase())):[])
async function setCommon(id:string,enabled:boolean){if(commonSaving.value)return;commonSaving.value=true;try{const current=store.settings.common_launchers??commonLaunchers.map(x=>x.id);await store.setSetting('common_launchers',enabled?[...new Set([...current,id])]:current.filter(x=>x!==id))}catch{}finally{commonSaving.value=false}}
const allCommonEnabled=computed(()=>enabledCommon.value.length===commonLaunchers.length)
async function setAllCommon(enabled:boolean){if(commonSaving.value)return;commonSaving.value=true;try{await store.setSetting('common_launchers',enabled?commonLaunchers.map(item=>item.id):[])}catch{}finally{commonSaving.value=false}}
async function launchCommon(id:string){if(commonOpening.has(id))return;commonOpening.add(id);try{await api.launchCommon(id)}catch(e){store.notify(String(e))}finally{commonOpening.delete(id)}}
const fixedDialogOpen=ref(false)
const fixedIds=computed(()=>fixedResourceIds(store.settings.client))
async function setFixed(id:string,enabled:boolean){try{await store.saveClient('lumo-launcher-fixed-items',enabled?[...new Set([...fixedIds.value,id])]:fixedIds.value.filter(value=>value!==id))}catch(e){store.notify(String(e))}}
const activeFilter=ref('all')
const displayMode=ref<'grid'|'list'>(store.settings.client?.['lumo-library-display']==='list'?'list':'grid')
watch(displayMode,value=>void store.saveClient('lumo-library-display',value).catch(e=>store.notify(String(e))))
const launcherLayout=computed(()=>displayMode.value==='grid'?(density.value==='compact'?'grid min-w-0 grid-cols-[repeat(auto-fill,minmax(6rem,1fr))] gap-2':density.value==='comfortable'?'grid min-w-0 grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-4':'grid min-w-0 grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-4'):'flex min-w-0 flex-col gap-2')
const launcherTile=computed(()=>displayMode.value==='grid'?'h-28 w-full min-w-0 flex-col items-center justify-center gap-3 rounded-2xl px-3 py-3 has-[>svg]:px-3':'min-h-16 h-auto w-full min-w-0 justify-start gap-3 rounded-xl px-4 py-3')
const launcherIcon=computed(()=>displayMode.value==='grid'?'size-10 shrink-0 object-contain':'size-8 shrink-0 object-contain')
watch(()=>store.groups,()=>{if(activeFilter.value.startsWith('group:')&&!store.groups.some(g=>activeFilter.value===`group:${g.id}`))activeFilter.value='all'})
const activeGroup=computed(()=>store.groups.find(group=>activeFilter.value===`group:${group.id}`))
const query=ref('')
const organizerOpen=ref(false), formSaving=ref(false), importing=ref(false)
const density=ref(String(store.settings.client?.['lumo-library-density']??'standard'))
const listColumns=ref<string[]>((store.settings.client?.['lumo-library-columns'] as string[]|undefined)??['group'])
const listColumnOptions=[{id:'path',name:'启动路径'},{id:'developer',name:'开发者'},{id:'group',name:'分组'},{id:'recent',name:'最近使用'}]
function toggleColumn(id:string,enabled:boolean){listColumns.value=enabled?[...new Set([...listColumns.value,id])]:listColumns.value.filter(value=>value!==id)}
watch(density,value=>void store.saveClient('lumo-library-density',value).catch(e=>store.notify(String(e))))
watch(listColumns,value=>void store.saveClient('lumo-library-columns',value).catch(e=>store.notify(String(e))),{deep:true})
const customOrder=ref<string[]>((store.settings.client?.['lumo-library-order'] as string[]|undefined)??[])
const sessionOrder=ref<string[]>([])
const draggingTile=ref(''), tileDrop=ref(''), groupDrop=ref(''), reordering=ref(false)
const previewOrder=ref<string[]|null>(null)
const customIcon=ref<string|null>(null), argumentsText=ref(''), workingDirectory=ref(''), allowDuplicate=ref(false)
const remoteDuplicate=ref<string|null>(null)
const duplicate=computed(()=>store.resources.find(r=>r.id===remoteDuplicate.value)||store.resources.find(r=>r.id!==editingId.value&&targetKey(r.kind,r.target)===targetKey(form.kind,form.target)))
async function chooseIcon(){const path=await open({filters:[{name:'PNG 图标',extensions:['png']}]});if(typeof path==='string')try{customIcon.value=await api.readIcon(path)}catch(e){store.notify(String(e))}}
async function chooseWorkingDirectory(){const path=await open({directory:true});if(typeof path==='string')workingDirectory.value=path}
const dropPreview=ref<{name:string;target:string;kind:ResourceKind;icon_data:string|null;selected:boolean}[]>([]), showDrop=ref(false), dropBusy=ref(false), dropHint=ref(false)
async function previewPaths(paths:string[]){try{dropPreview.value=(await api.previewTargets(paths)).map(item=>({...item,selected:!store.resources.some(r=>targetKey(r.kind,r.target)===targetKey(item.kind,item.target))}));showDrop.value=true}catch(e){store.notify(String(e))}}
async function importDrop(){if(dropBusy.value)return;dropBusy.value=true;let added=0,failed=0;try{for(const item of dropPreview.value.filter(i=>i.selected)){try{await api.createResource({...item,group_id:activeGroup.value?.id??null,aliases:''});item.selected=false;added++}catch(e){failed++;store.notify(`${item.name}：${String(e)}`)}}await store.reload();if(!failed)showDrop.value=false;store.notify(`新增 ${added} 项，失败 ${failed} 项`)}finally{dropBusy.value=false}}
function dropText(event:DragEvent){const text=event.dataTransfer?.getData('text/uri-list')||event.dataTransfer?.getData('text/plain');if(text&&/^https?:\/\//i.test(text.trim())){addResource();form.kind='web';form.target=text.trim();try{form.name=new URL(form.target).hostname}catch{}}}
let stopDrop:(()=>void)|undefined
let draggedResource:{id:string;x:number;y:number;active:boolean}|null=null
function beginResourceDrag(event:PointerEvent,id:string){if(event.button!==0||reordering.value)return;event.preventDefault();previewOrder.value=allOrderedIds();draggedResource={id,x:event.clientX,y:event.clientY,active:false}}
function resourcePointerMove(event:PointerEvent){if(!draggedResource)return;if(Math.hypot(event.clientX-draggedResource.x,event.clientY-draggedResource.y)>6){draggedResource.active=true;draggingTile.value=draggedResource.id;const target=document.elementFromPoint(event.clientX,event.clientY);const targetId=target?.closest<HTMLElement>('[data-resource-tile]')?.dataset.resourceTile??'';tileDrop.value=targetId;groupDrop.value=target?.closest<HTMLElement>('[data-resource-group]')?.dataset.resourceGroup??'';if(targetId&&targetId!==draggedResource.id&&previewOrder.value){const order=previewOrder.value.filter(id=>id!==draggedResource!.id);const index=order.indexOf(targetId);if(index>=0){order.splice(index,0,draggedResource.id);previewOrder.value=order}}}}
async function saveOrder(ids:string[]){if(reordering.value)return;reordering.value=true;try{await store.saveClient('lumo-library-order',ids);customOrder.value=ids;sortMode.value='custom'}finally{reordering.value=false}}
function allOrderedIds(){return [...new Set([...(sortMode.value==='custom'?customOrder.value:sessionOrder.value),...store.resources.map(i=>i.id)])]}
async function moveTile(id:string,direction:number){const visible=shownResources.value.map(i=>i.id),target=visible[visible.indexOf(id)+direction];if(!target)return;const ids=allOrderedIds();const from=ids.indexOf(id),to=ids.indexOf(target);[ids[from],ids[to]]=[ids[to]!,ids[from]!];try{await saveOrder(ids)}catch(e){store.notify(String(e))}}
async function resourcePointerUp(event:PointerEvent){const dragged=draggedResource;const order=previewOrder.value;const target=document.elementFromPoint(event.clientX,event.clientY);const group=target?.closest<HTMLElement>('[data-resource-group]');cancelResourceDrag();if(!dragged?.active)return;try{if(group){await api.moveResource(dragged.id,group.dataset.resourceGroup||null);await store.reload();return}const id=target?.closest<HTMLElement>('[data-resource-tile]')?.dataset.resourceTile;if(id&&id!==dragged.id&&order)await saveOrder(order)}catch(e){store.notify(String(e))}}
function cancelResourceDrag(){draggedResource=null;draggingTile.value='';tileDrop.value='';groupDrop.value='';previewOrder.value=null}
onMounted(async()=>{window.addEventListener('pointermove',resourcePointerMove);window.addEventListener('pointerup',resourcePointerUp);window.addEventListener('blur',cancelResourceDrag);window.addEventListener('pointercancel',cancelResourceDrag);try{const stop=await getCurrentWindow().onDragDropEvent(({payload})=>{dropHint.value=payload.type==='enter'||payload.type==='over';if(payload.type==='drop')void previewPaths(payload.paths)});stopDrop=stop}catch{}})
onUnmounted(()=>{stopDrop?.();window.removeEventListener('pointermove',resourcePointerMove);window.removeEventListener('pointerup',resourcePointerUp);window.removeEventListener('blur',cancelResourceDrag);window.removeEventListener('pointercancel',cancelResourceDrag)})
async function refreshHealth(){try{resourceHealth.value=await api.resourceHealth()}catch(e){store.notify(String(e))}}
const showForm=ref(false)
const editingId=ref('')
const groupManagerOpen=ref(false)
const showScan=ref(false)
const candidates=ref<ScanCandidate[]>([])
const scanCategory=ref('all')
const scanQuery=ref('')
const showImported=ref(false)
const resourceHealth=ref<Record<string,boolean>>({})
watch(()=>store.resources.map(item=>item.id+item.target).join('|'),async()=>{try{resourceHealth.value=await api.resourceHealth()}catch{}},{immediate:true})
const sortMode=ref(String(store.settings.client?.['lumo-library-sort']||'frequent')||'frequent')
watch(sortMode,value=>void writeLocal('lumo-library-sort',value).catch(e=>store.notify(String(e))))
const scanCategories=[{id:'all',label:'全部'},{id:'application',label:'第三方应用'},{id:'system',label:'系统自带应用'},{id:'extension',label:'扩展 / 插件应用'}]
const filteredCandidates=computed(()=>candidates.value.filter(item=>(scanCategory.value==='all'||item.category===scanCategory.value)&&(showImported.value||!item.imported)&&item.name.toLocaleLowerCase().includes(scanQuery.value.trim().toLocaleLowerCase())))
const publishers=ref<Record<string,string>>({})
let publisherVersion=0
watch([displayMode,()=>store.resources.map(item=>item.id+item.target).join('|')],async()=>{if(displayMode.value!=='list')return;const version=++publisherVersion;try{const result=await api.resourcePublishers();if(version===publisherVersion)publishers.value=result}catch{publishers.value={}}},{immediate:true})
const selectedTargets=ref<string[]>([])
const allScanSelected=computed(()=>filteredCandidates.value.length>0&&filteredCandidates.value.every(item=>selectedTargets.value.includes(item.target)))
function toggleAllScan(){const targets=new Set(filteredCandidates.value.map(item=>item.target));selectedTargets.value=allScanSelected.value?selectedTargets.value.filter(target=>!targets.has(target)):[...new Set([...selectedTargets.value,...targets])]}
const scanBusy=ref(false)
const focusedId=ref('')
const openingIds=reactive(new Set<string>())
const form=reactive({name:'',kind:'app' as ResourceKind,target:'',group_id:'',aliases:''})
watch([()=>form.kind,()=>form.target],()=>{if(form.kind!=='app'||!/\.exe$/i.test(form.target)){argumentsText.value='';workingDirectory.value=''}})
let duplicateTimer:ReturnType<typeof setTimeout>|undefined,duplicateVersion=0
watch([()=>form.kind,()=>form.target,editingId],()=>{remoteDuplicate.value=null;const version=++duplicateVersion;if(duplicateTimer)clearTimeout(duplicateTimer);if(!form.target.trim())return;duplicateTimer=setTimeout(async()=>{try{const id=await api.findDuplicate(form.kind,form.target,editingId.value||null);if(version===duplicateVersion)remoteDuplicate.value=id}catch{}},250)})
onUnmounted(()=>{if(duplicateTimer)clearTimeout(duplicateTimer);++duplicateVersion})
const categoryCounts=computed<Record<string,number>>(()=>{
  const items=store.resources
  const common=enabledCommon.value.length
  return {all:items.length+common,app:items.filter(item=>item.kind==='app').length,web:items.filter(item=>item.kind==='web').length,file:items.filter(item=>item.kind==='file').length,favorites:items.filter(item=>item.is_favorite).length,common,pinned:items.filter(item=>fixedIds.value.includes(item.id)).length}
})
function categoryCount(id:string){const count=categoryCounts.value[id]??0;return count?`(${count})`:''}
const filters=[{id:'all',label:'全部',icon:LayoutGrid},{id:'app',label:'应用',icon:AppWindow},{id:'web',label:'网页',icon:Globe},{id:'file',label:'文件',icon:FileText}] as const
watch(()=>store.searchTarget,async target=>{if(target?.kind==='resource'){focusedId.value='';activeFilter.value='all';await nextTick();document.getElementById(`resource-${target.id}`)?.scrollIntoView({behavior:'smooth',block:'center'});store.searchTarget=null}},{immediate:true})
const matchingIds=ref<string[]>([])
let searchVersion=0
watch([query,()=>store.resources],async ([value])=>{const version=++searchVersion;matchingIds.value=[];try{const results=await api.searchResources(value);if(version===searchVersion)matchingIds.value=results.map(item=>item.id)}catch(e){store.notify(String(e))}})
const shownResources=computed(()=>{
  const list=store.resources.filter(r=>(activeFilter.value==='all'||(activeFilter.value==='pinned'&&fixedIds.value.includes(r.id))||activeFilter.value===r.kind||(activeFilter.value==='favorites'&&r.is_favorite)||activeFilter.value===`group:${r.group_id}`)&&(!query.value.trim()||matchingIds.value.includes(r.id)))
  const sorted=list.sort((a,b)=>query.value.trim()?matchingIds.value.indexOf(a.id)-matchingIds.value.indexOf(b.id):sortMode.value==='name'?a.name.localeCompare(b.name):sortMode.value==='recent'?(b.last_opened_at??'').localeCompare(a.last_opened_at??'')||a.name.localeCompare(b.name):b.usage_count-a.usage_count||Number(b.is_favorite)-Number(a.is_favorite)||a.name.localeCompare(b.name))
  if(query.value.trim())return sorted
  return stableResourceOrder(sorted,previewOrder.value??(sortMode.value==='custom'?customOrder.value:sessionOrder.value))
})
watch([sortMode,activeFilter,query],()=>{sessionOrder.value=[]})
watch(shownResources,items=>{if(!query.value.trim()&&!sessionOrder.value.length&&items.length)sessionOrder.value=items.map(i=>i.id)},{immediate:true})
const batchMode=ref(false), selectedIds=ref<string[]>([]), batchBusy=ref(false), deleteBatch=ref(false)
watch([activeFilter,query],()=>{selectedIds.value=[]})
function toggleSelection(id:string){selectedIds.value=selectedIds.value.includes(id)?selectedIds.value.filter(x=>x!==id):[...selectedIds.value,id]}
async function batch(action:'move'|'favorite'|'unfavorite'|'delete',groupId:string|null=null,ids=selectedIds.value){
  batchBusy.value=true
  try{await api.batchResources(ids,action,groupId);await store.reload();selectedIds.value=[];store.notify('操作完成')}catch(e){store.notify(String(e))}finally{batchBusy.value=false}
}
async function reveal(item:Resource){try{await api.revealResource(item.id)}catch(e){store.notify(String(e))}}
async function copyPath(item:Resource){try{await navigator.clipboard.writeText(item.target);store.notify('路径已复制')}catch(e){store.notify(`复制失败：${String(e)}`)}}

function clearForm(){showForm.value=false;editingId.value='';form.name='';form.kind='app';form.target='';form.group_id='';form.aliases='';argumentsText.value='';workingDirectory.value='';customIcon.value=null;allowDuplicate.value=false}
function addResource(){clearForm();showForm.value=true}
function newLibraryItem(){addResource()}
onMounted(()=>window.addEventListener('lumo-new',newLibraryItem))
onUnmounted(()=>window.removeEventListener('lumo-new',newLibraryItem))
function edit(item:Resource){editingId.value=item.id;form.name=item.name;form.kind=item.kind;form.target=item.target;form.group_id=item.group_id??'';form.aliases=item.aliases;const options=launchOptions(store.settings.client,item.id);argumentsText.value=options.arguments.join('\n');workingDirectory.value=options.working_directory;customIcon.value=options.icon_data;allowDuplicate.value=false;showForm.value=true}
async function locateDuplicate(){const id=duplicate.value?.id;if(!id)return;clearForm();query.value='';activeFilter.value='all';focusedId.value=id;await nextTick();document.getElementById(`resource-${id}`)?.scrollIntoView({block:'center',behavior:'smooth'})}
async function pickTarget(){const result=await open({multiple:false,directory:false,filters:form.kind==='app'?[{name:'应用程序或快捷方式',extensions:['exe','lnk']}]:undefined});if(typeof result==='string'){try{const [item]=await api.previewTargets([result]);if(item){form.target=item.target;form.kind=item.kind;if(!form.name.trim())form.name=item.name}}catch(e){store.notify(String(e))}}}
async function save(){
  if(formSaving.value)return
  if(!form.name.trim()||!form.target.trim()){store.notify('请填写名称和启动目标');return}
  formSaving.value=true
  try{
    const foundId=await api.findDuplicate(form.kind,form.target,editingId.value||null);remoteDuplicate.value=foundId
    if(duplicate.value&&!allowDuplicate.value){store.notify('目标已存在，请定位已有记录或勾选创建另一个入口');return}
    let id=editingId.value
    const value={name:form.name.trim(),kind:form.kind,target:form.target.trim(),group_id:form.group_id||null,aliases:form.aliases.trim()}
    if(id){const old=store.resources.find(x=>x.id===id);if(!old)throw new Error('启动项已不存在');await api.updateResource({...old,...value});Object.assign(old,value)}
    else {const item=await api.createResource({...value,allow_duplicate:allowDuplicate.value});id=item.id;editingId.value=id;store.resources.push(item)}
    await api.saveLaunchOptions(id,{arguments:argumentsText.value.split('\n').filter(line=>line.length>0),working_directory:workingDirectory.value.trim(),icon_data:customIcon.value})
    await store.reload();store.notify('启动项已保存');clearForm()
  }catch(e){store.notify(`保存失败：${String(e)}`)}finally{formSaving.value=false}
}
async function remove(item:Resource){if(!await askConfirmation(`从应用库移除“${item.name}”？原始程序或文件不会删除。`))return;try{await api.deleteResource(item.id);await store.reload();store.notify('已从应用库移除')}catch(e){store.notify(String(e))}}
async function launch(item:Resource){if(openingIds.has(item.id))return;if(resourceHealth.value[item.id]===false){await relocate(item);return;}openingIds.add(item.id);try{await store.launchResource(item.id)}catch(e){store.notify(String(e));if(item.kind!=='web'&&await askConfirmation('该路径可能已失效。现在重新定位吗？'))await relocate(item)}finally{openingIds.delete(item.id)}}
async function relocate(item:Resource){const result=await open({multiple:false,directory:false,filters:item.kind==='app'?[{name:'应用程序或快捷方式',extensions:['exe','lnk']}]:undefined});if(typeof result!=='string')return;try{await api.relocateResource(item.id,result);await store.reload();store.notify('路径已更新')}catch(e){store.notify(String(e))}}
async function favorite(item:Resource){try{await api.toggleFavorite(item.id);await store.reload()}catch(e){store.notify(String(e))}}
async function scan(){scanBusy.value=true;showScan.value=true;try{candidates.value=await api.scanApplications();selectedTargets.value=[];scanCategory.value='application';scanQuery.value='';showImported.value=false}catch(e){store.notify(`扫描失败：${String(e)}`)}finally{scanBusy.value=false}}
async function importSelected(){if(importing.value)return;const selected=candidates.value.filter(c=>selectedTargets.value.includes(c.target));if(!selected.length)return;importing.value=true;try{const before=store.resources.length;await api.importCandidates(selected);await store.reload();const count=store.resources.length-before;showScan.value=false;store.notify(`新增 ${count} 项，已存在 ${selected.length-count} 项`)}catch(e){store.notify(`导入失败：${String(e)}`)}finally{importing.value=false}}
function iconFor(kind:ResourceKind){return kind==='app'?AppWindow:kind==='web'?Globe:FileText}
</script>

<template>
  <div class="flex h-full min-h-0 flex-1 flex-col overflow-hidden" @dragover.prevent @drop.prevent="dropText"><LibraryManagement v-model:open="organizerOpen" :health="resourceHealth" @relocate="relocate" @refresh="refreshHealth"/><GroupManagement v-model:open="groupManagerOpen"/><p v-if="dropHint" role="status" class="rounded-lg bg-primary/10 p-3 text-sm text-primary">松开以预览并添加文件或应用</p><Dialog v-model:open="showDrop"><DialogContent class="flex max-h-[85dvh] flex-col"><DialogHeader><DialogTitle>添加拖入的启动项</DialogTitle><DialogDescription>勾选需要添加的项目；已存在的目标默认跳过。</DialogDescription></DialogHeader><div class="min-h-0 overflow-y-auto"><label v-for="item in dropPreview" :key="item.target" class="flex items-center gap-3 py-2"><Checkbox v-model="item.selected" :disabled="dropBusy"/><img v-if="item.icon_data" :src="item.icon_data" class="size-8 object-contain" alt=""/><span class="min-w-0 flex-1"><span class="block truncate">{{ item.name }}</span><span class="block truncate text-xs text-muted-foreground">{{ item.target }}</span></span></label></div><DialogFooter><Button variant="outline" :disabled="dropBusy" @click="showDrop=false">取消</Button><Button :disabled="dropBusy||!dropPreview.some(i=>i.selected)" @click="importDrop">{{ dropBusy?'正在添加…':'添加所选' }}</Button></DialogFooter></DialogContent></Dialog>
    <Dialog v-model:open="commonDialogOpen"><DialogContent class="flex max-h-[85dvh] flex-col"><DialogHeader><DialogTitle>配置系统启动项</DialogTitle><DialogDescription>开启的项目显示在「全部」和「系统启动项」中，修改后自动保存。</DialogDescription></DialogHeader><FieldGroup class="min-h-0 gap-3 overflow-y-auto p-1"><Field orientation="horizontal"><FieldLabel for="common-all">全部开启 / 全部关闭</FieldLabel><Switch id="common-all" :model-value="allCommonEnabled" :disabled="commonSaving" @update:model-value="setAllCommon" /></Field><Field v-for="item in commonLaunchers" :key="item.id" orientation="horizontal"><FieldLabel :for="`common-${item.id}`"><img v-if="commonIcons[item.id]" :src="commonIcons[item.id]" alt="" class="size-5 shrink-0 object-contain" /><component :is="item.icon" v-else class="size-5 shrink-0" />{{ item.name }}</FieldLabel><Switch :id="`common-${item.id}`" :model-value="enabledCommon.some(x=>x.id===item.id)" :disabled="commonSaving" @update:model-value="value=>setCommon(item.id,value)" /></Field></FieldGroup></DialogContent></Dialog>
    <Dialog :open="showForm" @update:open="value=>{if(!value&&!formSaving)clearForm()}"><DialogContent class="flex max-h-[85dvh] flex-col sm:max-w-xl">
      <DialogHeader><DialogTitle>{{ editingId?'编辑启动项':'添加启动项' }}</DialogTitle><DialogDescription>填写启动项信息，将应用、网页或文件添加到应用库。</DialogDescription></DialogHeader>
      <form class="flex min-h-0 flex-col gap-5" @submit.prevent="save">
      <fieldset :disabled="formSaving" class="min-h-0 overflow-y-auto p-1"><FieldGroup>
        <Field><FieldLabel for="resource-name">名称</FieldLabel><Input id="resource-name" v-model="form.name" placeholder="例如：浏览器、项目文件" /></Field>
        <Field><FieldLabel for="resource-kind">类型</FieldLabel><Select v-model="form.kind"><SelectTrigger id="resource-kind" class="w-full"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem value="app">应用</SelectItem><SelectItem value="web">网页</SelectItem><SelectItem value="file">文件</SelectItem></SelectGroup></SelectContent></Select></Field>
        <Field><FieldLabel for="resource-target">{{ form.kind==='web'?'网址':'程序或文件路径' }}</FieldLabel><div class="flex gap-2"><Input id="resource-target" v-model="form.target" :placeholder="form.kind==='web'?'https://example.com':'选择或输入本地路径'" /><Button v-if="form.kind!=='web'" type="button" variant="outline" @click="pickTarget">浏览</Button></div></Field>
        <Field><FieldLabel for="resource-aliases">搜索别名</FieldLabel><Input id="resource-aliases" v-model="form.aliases" placeholder="例如：浏览器 chrome 谷歌" /></Field>
        <Field><FieldLabel for="resource-group">分组</FieldLabel><Select :model-value="form.group_id || 'ungrouped'" @update:model-value="value=>form.group_id=String(value)==='ungrouped'?'':String(value)"><SelectTrigger id="resource-group" class="w-full"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem value="ungrouped">未分组</SelectItem><SelectItem v-for="group in store.groups" :key="group.id" :value="group.id">{{ group.name }}</SelectItem></SelectGroup></SelectContent></Select></Field>
        <Field><FieldLabel>自定义图标</FieldLabel><div class="flex items-center gap-2"><img v-if="customIcon" :src="customIcon" alt="" class="size-10 object-contain"/><Button type="button" variant="outline" @click="chooseIcon">选择 PNG</Button><Button type="button" variant="ghost" @click="customIcon=null">恢复默认</Button></div></Field>
        <details v-if="form.kind==='app'&&/\.exe$/i.test(form.target)" class="text-sm"><summary>高级启动选项</summary><FieldGroup class="mt-3"><Field><FieldLabel for="launch-arguments">启动参数（每行一个参数，不需要外层引号）</FieldLabel><Textarea id="launch-arguments" v-model="argumentsText" placeholder="--profile&#10;work"/></Field><Field><FieldLabel for="working-directory">工作目录</FieldLabel><div class="flex gap-2"><Input id="working-directory" v-model="workingDirectory"/><Button type="button" variant="outline" @click="chooseWorkingDirectory">浏览</Button></div></Field></FieldGroup></details>
        <div v-if="duplicate" class="flex flex-col gap-2 text-sm"><p>已存在：{{ duplicate.name }}</p><Button type="button" variant="outline" @click="locateDuplicate">定位已有记录</Button><label class="flex items-center gap-2"><Checkbox v-model="allowDuplicate"/>创建另一个入口</label></div>
      </FieldGroup></fieldset>
      <DialogFooter class="shrink-0"><Button type="button" variant="outline" :disabled="formSaving" @click="clearForm">取消</Button><Button type="submit" :disabled="formSaving">{{ formSaving?'保存中…':'保存启动项' }}</Button></DialogFooter>
      </form>
    </DialogContent></Dialog>
    <Dialog v-model:open="showScan"><DialogContent class="flex max-h-[85dvh] flex-col sm:max-w-3xl">
      <DialogHeader><DialogTitle>扫描本机应用</DialogTitle><DialogDescription>识别本机程序并按程序路径去重，只导入勾选的应用。</DialogDescription></DialogHeader>
      <Tabs v-if="!scanBusy" v-model="scanCategory" class="shrink-0"><TabsList class="h-auto flex-wrap" aria-label="扫描应用分类"><TabsTrigger v-for="category in scanCategories" :key="category.id" :value="category.id">{{ category.label }} · {{ candidates.filter(item=>category.id==='all'||item.category===category.id).length }}</TabsTrigger></TabsList></Tabs>
      <Field v-if="!scanBusy" orientation="horizontal"><Checkbox id="show-imported" v-model="showImported" /><FieldLabel for="show-imported">显示已添加的应用（{{ candidates.filter(item=>item.imported).length }}）</FieldLabel></Field>
      <Input v-if="!scanBusy" v-model="scanQuery" class="shrink-0" placeholder="搜索应用名称" aria-label="搜索扫描到的应用名称" />
      <p v-if="!scanBusy" role="status" class="text-sm text-muted-foreground">显示 {{ filteredCandidates.length }} / {{ candidates.length }} 个应用 · 已选择 {{ selectedTargets.length }} 个</p>
      <p v-if="scanBusy" class="py-6 text-center text-sm text-muted-foreground">正在扫描……</p>
      <div v-else-if="filteredCandidates.length" class="flex min-h-0 max-h-[55dvh] flex-col gap-2 overflow-y-auto overscroll-contain rounded-2xl border border-border p-3">
        <label v-for="item in filteredCandidates" :key="item.target" class="flex min-w-0 items-center gap-3 rounded-xl px-3 py-3 hover:bg-accent">
          <Checkbox :model-value="selectedTargets.includes(item.target)" :aria-label="`选择 ${item.name}`" @update:model-value="value=>selectedTargets=value===true?[...selectedTargets,item.target]:selectedTargets.filter(target=>target!==item.target)" />
          <img v-if="item.icon_data" :src="item.icon_data" alt="" class="size-8 shrink-0 object-contain" /><AppWindow v-else class="size-8 shrink-0 text-muted-foreground" />
          <span class="flex min-w-0 flex-1 flex-col gap-1">
            <span class="flex items-center gap-3"><span class="truncate text-sm font-medium">{{ item.name }}</span><Badge v-if="item.imported" variant="secondary">已添加</Badge><span class="shrink-0 text-xs text-muted-foreground">{{ item.source }}</span></span>
            <span class="truncate text-xs text-muted-foreground" :title="item.installation_path??item.target">安装位置：{{ item.installation_path??'未能识别' }}</span>
            <span class="truncate text-xs text-muted-foreground" :title="item.target">启动入口：{{ item.target }}</span>
          </span>
        </label>
      </div>
      <p v-else-if="!scanBusy" class="py-6 text-center text-sm text-muted-foreground">{{ candidates.length?'没有匹配的应用，请调整分类或搜索名称。':'没有找到可导入的应用。' }}</p>
      <DialogFooter class="shrink-0"><Button variant="outline" :disabled="scanBusy||!filteredCandidates.length" @click="toggleAllScan">{{ allScanSelected?'取消全选':'全选' }}</Button><Button variant="outline" @click="showScan=false">取消</Button><Button :disabled="scanBusy||importing||!selectedTargets.length" @click="importSelected">{{ importing?'导入中…':'导入所选' }}（{{ selectedTargets.length }}）</Button></DialogFooter>
    </DialogContent></Dialog>
    <Tabs v-model="activeFilter" class="flex min-h-0 min-w-0 flex-1 flex-col gap-4">
      <div class="flex min-w-0 shrink-0 flex-nowrap items-center gap-3">
        <div class="flex min-w-0 flex-1 flex-nowrap items-center gap-2 overflow-x-auto py-1" aria-label="应用库筛选和分组">
          <Select :model-value="activeFilter.startsWith('group:')?'all':activeFilter" @update:model-value="value=>activeFilter=String(value)"><SelectTrigger class="h-9 w-40 shrink-0" aria-label="应用库分类"><SelectValue/></SelectTrigger><SelectContent><SelectGroup><SelectItem v-for="item in filters" :key="item.id" :value="item.id"><component :is="item.icon"/>{{ item.label }}{{ categoryCount(item.id) }}</SelectItem><SelectItem value="favorites"><Star/>收藏{{ categoryCount('favorites') }}</SelectItem><SelectItem value="common"><Monitor/>系统启动项{{ categoryCount('common') }}</SelectItem><SelectItem value="pinned"><Star/>固定项{{ categoryCount('pinned') }}</SelectItem></SelectGroup></SelectContent></Select>
          <Select :model-value="activeGroup?.id??'none'" @update:model-value="value=>activeFilter=value==='none'?'all':`group:${value}`"><SelectTrigger class="h-9 w-40 shrink-0" aria-label="选择应用分组"><SelectValue placeholder="选择分组"/></SelectTrigger><SelectContent><SelectGroup><SelectItem value="none">所有分组</SelectItem><SelectItem v-for="group in store.groups" :key="group.id" :value="group.id" :data-resource-group="group.id">{{ group.name }}{{ store.settings.client?.['lumo-launcher-pinned-group']===group.id?' · 已固定':'' }}</SelectItem></SelectGroup></SelectContent></Select>
          <Button variant="ghost" size="icon-sm" aria-label="管理分组" @click="groupManagerOpen=true"><FolderCog/></Button>
        </div>
        <div class="ml-auto flex shrink-0 flex-nowrap items-center justify-end gap-2" aria-label="应用库操作">
          <Select v-model="sortMode"><SelectTrigger class="h-8 w-36" aria-label="应用排序"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem value="frequent">常用优先</SelectItem><SelectItem value="recent">最近使用</SelectItem><SelectItem value="name">名称排序</SelectItem><SelectItem value="custom">自定义排序</SelectItem></SelectGroup></SelectContent></Select>
          <DropdownMenu v-if="displayMode==='list'"><DropdownMenuTrigger as-child><Button variant="outline" size="sm">显示信息</Button></DropdownMenuTrigger><DropdownMenuContent><DropdownMenuGroup><DropdownMenuCheckboxItem v-for="column in listColumnOptions" :key="column.id" :model-value="listColumns.includes(column.id)" @select="$event.preventDefault()" @update:model-value="value=>toggleColumn(column.id,value===true)">{{ column.name }}</DropdownMenuCheckboxItem></DropdownMenuGroup></DropdownMenuContent></DropdownMenu><div class="relative w-48 shrink-0"><Search class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" /><Input v-model="query" class="h-8 pl-9" placeholder="名称、拼音或别名" aria-label="筛选启动项" /></div>
          <div class="flex shrink-0 items-center gap-px"><Button size="sm" class="rounded-r-none" @click="addResource"><Plus data-icon="inline-start" />添加</Button><DropdownMenu><DropdownMenuTrigger as-child><Button size="icon-sm" class="rounded-l-none" aria-label="更多应用库操作" title="更多操作"><Ellipsis /></Button></DropdownMenuTrigger><DropdownMenuContent align="end"><DropdownMenuGroup><DropdownMenuItem @select="organizerOpen=true"><Folder/>整理分组 / 失效检查 / 工作组合</DropdownMenuItem><DropdownMenuItem @select="batchMode=!batchMode;selectedIds=[]"><Pencil />{{ batchMode?'完成管理':'管理应用' }}</DropdownMenuItem><DropdownMenuItem :disabled="scanBusy" @select="scan"><RefreshCw />扫描本机应用</DropdownMenuItem><DropdownMenuItem @select="commonDialogOpen=true"><Settings />配置系统启动项</DropdownMenuItem></DropdownMenuGroup></DropdownMenuContent></DropdownMenu></div>
        </div>
      </div>
      <div v-if="activeFilter==='pinned'" class="flex items-center justify-between gap-2"><p class="text-sm text-muted-foreground">固定项为内置分类，显示在快捷启动面板中，不影响应用原分组。</p><Button size="sm" variant="outline" @click="fixedDialogOpen=true"><Plus/>添加固定项</Button></div>
      <TabsContent :value="activeFilter" class="flex min-h-0 flex-1 flex-col overflow-hidden"><ScrollArea class="library-scroll min-h-0 w-full flex-1" aria-label="应用库启动项"><div class="flex min-h-full min-w-0 flex-1 flex-col gap-4 pr-3 pb-3">
        <h2 v-if="shownCommon.length&&activeFilter==='all'" class="text-xs font-medium text-muted-foreground">系统启动项 · {{ shownCommon.length }}</h2><div v-if="shownCommon.length" :class="launcherLayout" aria-label="系统启动项"><Button v-for="(item,index) in shownCommon" :key="item.id" variant="ghost" :class="launcherTile" :title="item.name" :aria-busy="commonOpening.has(item.id)" :disabled="commonOpening.has(item.id)" @click="launchCommon(item.id)"><span v-if="displayMode==='list'" class="w-7 shrink-0 text-center text-sm tabular-nums text-muted-foreground">{{ index+1 }}</span><img v-if="commonIcons[item.id]" :src="commonIcons[item.id]" alt="" :class="launcherIcon" /><component :is="item.icon" v-else :class="[launcherIcon,'text-muted-foreground']" /><span class="min-w-0 truncate text-sm" :class="displayMode==='grid'?'w-full text-center':'flex-1 text-left'">{{ commonOpening.has(item.id)?'正在打开…':item.name }}</span><span v-if="displayMode==='list'&&listColumns.includes('developer')" class="w-32 shrink-0 truncate text-right text-sm text-muted-foreground">Microsoft</span></Button></div>
        <div v-if="batchMode&&activeFilter!=='common'" class="flex flex-wrap items-center gap-3 rounded-xl bg-muted p-3">
          <Checkbox :model-value="shownResources.length>0&&shownResources.every(item=>selectedIds.includes(item.id))" aria-label="选择当前结果" @update:model-value="value=>selectedIds=value===true?shownResources.map(item=>item.id):[]"/><span class="text-sm">已选 {{ selectedIds.length }} 项</span>
          <DropdownMenu><DropdownMenuTrigger as-child><Button variant="outline" size="sm" :disabled="!selectedIds.length||batchBusy">移动分组</Button></DropdownMenuTrigger><DropdownMenuContent><DropdownMenuGroup><DropdownMenuItem @select="batch('move',null)">未分组</DropdownMenuItem><DropdownMenuItem v-for="group in store.groups" :key="group.id" @select="batch('move',group.id)">{{ group.name }}</DropdownMenuItem></DropdownMenuGroup></DropdownMenuContent></DropdownMenu>
          <Button variant="outline" size="sm" :disabled="!selectedIds.length||batchBusy" @click="batch('favorite')">收藏</Button><Button variant="outline" size="sm" :disabled="!selectedIds.length||batchBusy" @click="batch('unfavorite')">取消收藏</Button><Button variant="destructive" size="sm" :disabled="!selectedIds.length||batchBusy" @click="deleteBatch=true">移除所选</Button>
        </div>
        <h2 v-if="shownCommon.length&&shownResources.length&&activeFilter==='all'" class="text-xs font-medium text-muted-foreground">我的启动项 · {{ shownResources.length }}</h2><p v-if="draggingTile" role="status" class="text-xs text-primary">拖到其他启动项前可排序，拖到分组标签可移动分组。</p><div v-if="shownResources.length" :class="batchMode?'grid grid-cols-[repeat(auto-fill,minmax(min(100%,22rem),1fr))] gap-4':launcherLayout">
          <ContextMenu v-for="(item,index) in shownResources" :key="item.id"><ContextMenuTrigger as-child><div class="group relative min-w-0 rounded-2xl" :class="tileDrop===item.id?'ring-2 ring-primary':draggingTile===item.id?'opacity-50':''" :data-resource-tile="item.id"><Button v-if="!batchMode" :id="`resource-${item.id}`" variant="ghost" :class="[launcherTile,focusedId===item.id?'ring-2 ring-primary':'']" :title="item.name" :aria-label="item.name" :aria-busy="openingIds.has(item.id)" :disabled="openingIds.has(item.id)" @click="launch(item)"><span v-if="displayMode==='list'" class="w-7 shrink-0 text-center text-sm tabular-nums text-muted-foreground">{{ shownCommon.length+index+1 }}</span><span class="relative shrink-0" :class="displayMode==='grid'?'size-10':'size-8'"><img v-if="resourceImage(store.settings.client,item)" :src="resourceImage(store.settings.client,item)!" alt="" :class="launcherIcon"/><component :is="iconFor(item.kind)" v-else :class="[launcherIcon,'text-primary']"/></span><span class="min-w-0" :class="displayMode==='grid'?'w-full text-center':'flex-1 text-left'"><span class="block truncate text-sm">{{ openingIds.has(item.id)?'正在打开…':item.name }}</span><span v-if="resourceHealth[item.id]===false" class="text-destructive" :class="displayMode==='grid'?'absolute inset-x-1 bottom-1 truncate text-xs':'block text-xs'">{{ displayMode==='grid'?'路径失效':'路径失效 · 点击重新定位' }}</span><span v-if="displayMode==='list'&&listColumns.includes('path')" class="mt-1 block truncate text-xs text-muted-foreground">{{ item.target }}</span><span v-if="displayMode==='list'&&(listColumns.includes('group')||listColumns.includes('recent'))" class="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground"><span v-if="listColumns.includes('group')">{{ store.groups.find(g=>g.id===item.group_id)?.name||'未分组' }}</span><span v-if="listColumns.includes('recent')">{{ item.last_opened_at?'最近使用：'+new Date(item.last_opened_at).toLocaleString('zh-CN'):'尚未使用' }}</span></span></span><span v-if="displayMode==='list'&&listColumns.includes('developer')" class="w-32 shrink-0 truncate text-right text-sm text-muted-foreground">{{ item.kind==='app'?(publishers[item.id]||'未知开发者'):'—' }}</span></Button><Card v-else :id="`resource-${item.id}`" :key="item.id" class="flex min-w-0 flex-col gap-4 p-5 transition-shadow" :class="focusedId===item.id?'ring-2 ring-primary':''">
            <div class="flex min-w-0 items-start gap-3"><Checkbox v-if="batchMode" :model-value="selectedIds.includes(item.id)" :aria-label="`选择 ${item.name}`" @update:model-value="toggleSelection(item.id)"/><div class="flex size-11 shrink-0 items-center justify-center rounded-2xl bg-primary/10 text-primary"><img v-if="resourceImage(store.settings.client,item)" :src="resourceImage(store.settings.client,item)!" alt="" class="size-8 object-contain" /><component :is="iconFor(item.kind)" v-else /></div><div class="min-w-0 flex-1"><h2 class="truncate font-semibold">{{ item.name }}</h2><p class="mt-1 truncate text-xs text-muted-foreground">{{ item.target }}</p></div><Button variant="ghost" size="icon" :aria-label="item.is_favorite?'取消收藏':'收藏'" @click="favorite(item)"><Heart :class="item.is_favorite?'fill-primary text-primary':'text-muted-foreground'" /></Button></div>
            <div class="flex flex-wrap items-center justify-between gap-2"><Badge>{{ item.kind==='app'?'应用':item.kind==='web'?'网页':'文件' }}</Badge><span class="flex gap-1"><Button v-if="item.kind!=='web'" variant="ghost" size="sm" @click="relocate(item)">重新定位</Button><Button variant="ghost" size="icon" aria-label="编辑" @click="edit(item)"><Pencil /></Button><Button variant="danger" size="icon" aria-label="删除" @click="remove(item)"><Trash2 /></Button><Button variant="outline" size="sm" @click="launch(item)">打开<ArrowUpRight data-icon="inline-end" /></Button></span></div>
          </Card><template v-if="!batchMode"><Button variant="ghost" size="icon-xs" class="absolute left-1 top-1 touch-none cursor-grab opacity-0 group-hover:opacity-100 focus-visible:opacity-100" aria-label="拖动排序或移入分组" @pointerdown="beginResourceDrag($event,item.id)"><GripVertical/></Button></template></div></ContextMenuTrigger>
          <ContextMenuContent class="w-52"><ContextMenuGroup>
            <ContextMenuItem @select="launch(item)">打开</ContextMenuItem><ContextMenuItem :disabled="reordering||index===0" @select="moveTile(item.id,-1)">向前移动</ContextMenuItem><ContextMenuItem :disabled="reordering||index===shownResources.length-1" @select="moveTile(item.id,1)">向后移动</ContextMenuItem><ContextMenuItem :disabled="item.kind==='web'" @select="reveal(item)">打开所在文件夹</ContextMenuItem><ContextMenuItem @select="copyPath(item)">复制{{ item.kind==='web'?'网址':'路径' }}</ContextMenuItem><ContextMenuItem @select="favorite(item)">{{ item.is_favorite?'取消收藏':'收藏' }}</ContextMenuItem>
            <ContextMenuSub><ContextMenuSubTrigger>移动分组</ContextMenuSubTrigger><ContextMenuSubContent><ContextMenuGroup><ContextMenuItem @select="batch('move',null,[item.id])">未分组</ContextMenuItem><ContextMenuItem v-for="group in store.groups" :key="group.id" @select="batch('move',group.id,[item.id])">{{ group.name }}</ContextMenuItem></ContextMenuGroup></ContextMenuSubContent></ContextMenuSub>
            <ContextMenuItem @select="setFixed(item.id,!fixedIds.includes(item.id))">{{ fixedIds.includes(item.id)?'从固定项移除':'添加到固定项' }}</ContextMenuItem><ContextMenuItem @select="edit(item)">编辑名称、别名和路径</ContextMenuItem><ContextMenuItem @select="batchMode=true;toggleSelection(item.id)">选择 / 取消选择</ContextMenuItem>
          </ContextMenuGroup><ContextMenuSeparator/><ContextMenuGroup><ContextMenuItem variant="destructive" @select="remove(item)">从应用库移除</ContextMenuItem></ContextMenuGroup></ContextMenuContent></ContextMenu>
        </div>
        <Card v-else-if="!shownCommon.length" class="flex min-h-64 flex-1 flex-col items-center justify-center p-8 text-center"><AppWindow class="mb-4 text-primary" /><h2 class="font-semibold">{{ activeFilter==='common'?'没有启用的系统启动项':store.resources.length?'没有匹配的启动项':'还没有启动项' }}</h2><p class="mt-2 max-w-sm text-sm text-muted-foreground">{{ activeFilter==='common'?'在配置中开启系统启动项，或调整搜索关键词。':store.resources.length?'试试切换筛选条件，或调整搜索关键词。':'手动添加应用、网页或文件，也可以扫描常见已安装应用并选择导入。' }}</p><Button v-if="activeFilter==='common'" class="mt-5" variant="outline" @click="commonDialogOpen=true"><Settings data-icon="inline-start" />配置系统启动项</Button><div v-else class="mt-5 flex gap-2"><Button variant="outline" @click="addResource"><Plus data-icon="inline-start"/>添加启动项</Button></div></Card>
        </div></ScrollArea>
      </TabsContent>
    </Tabs>
  </div>
  <Dialog v-model:open="fixedDialogOpen"><DialogContent><DialogHeader><DialogTitle>添加固定项</DialogTitle><DialogDescription>勾选应用库中的启动项，将其添加到快捷启动的固定项。</DialogDescription></DialogHeader><div class="flex max-h-96 flex-col gap-3 overflow-y-auto"><label v-for="item in store.resources" :key="item.id" class="flex items-center gap-3"><Checkbox :model-value="fixedIds.includes(item.id)" @update:model-value="value=>setFixed(item.id,value===true)"/><span>{{ item.name }}</span></label><p v-if="!store.resources.length" class="text-sm text-muted-foreground">请先添加启动项到应用库。</p></div></DialogContent></Dialog>
  <ConfirmDialog v-model:open="deleteBatch" title="移除所选启动项？" description="只移除应用库中的记录，不会删除电脑中的原始程序或文件。" confirm-text="移除" destructive @confirm="batch('delete')" />
</template>


<style scoped>
.library-scroll :deep([data-slot="scroll-area-viewport"] > div) {
  display: flex !important;
  min-height: 100%;
  flex-direction: column;
}
</style>
