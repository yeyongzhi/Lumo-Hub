<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { api } from '@/lib/api'
import { useWorkspaceStore } from '@/stores/workspace'
import type { LaunchScene, Resource } from '@/lib/types'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Checkbox } from '@/components/ui/checkbox'
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '@/components/ui/dialog'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { GripVertical, ArrowUp, ArrowDown, Pencil, Trash2 } from '@lucide/vue'
import { askConfirmation } from '@/lib/confirm'
const props = defineProps<{ health: Record<string,boolean> }>()
const emit = defineEmits<{ relocate: [item:Resource]; refresh: [] }>()
const opened = defineModel<boolean>('open', { default: false })
const store = useWorkspaceStore(), tab = ref('groups'), busy = ref(false)
const names = ref<Record<string,string>>({})
let groupDrag:{id:string;pointer:number;x:number;y:number}|null=null
const draggingGroup=ref(''), groupDrop=ref('')
function startGroupDrag(event:PointerEvent,id:string){if(event.button!==0||busy.value)return;event.preventDefault();groupDrag={id,pointer:event.pointerId,x:event.clientX,y:event.clientY}}
function groupPointerMove(event:PointerEvent){if(!groupDrag||event.pointerId!==groupDrag.pointer)return;if(Math.hypot(event.clientX-groupDrag.x,event.clientY-groupDrag.y)>5){draggingGroup.value=groupDrag.id;groupDrop.value=document.elementFromPoint(event.clientX,event.clientY)?.closest<HTMLElement>('[data-group-order]')?.dataset.groupOrder??''}}
async function finishGroupDrag(event:PointerEvent){if(!groupDrag||event.pointerId!==groupDrag.pointer)return;const id=draggingGroup.value,target=groupDrop.value;cancelGroupDrag();if(!id||!target||id===target)return;await operation(async()=>{const ids=store.groups.map(g=>g.id).filter(i=>i!==id);ids.splice(ids.indexOf(target),0,id);await api.reorderGroups(ids);store.groups=await api.listGroups()})}
function cancelGroupDrag(){groupDrag=null;draggingGroup.value='';groupDrop.value=''}
onMounted(()=>{window.addEventListener('pointermove',groupPointerMove);window.addEventListener('pointerup',finishGroupDrag);window.addEventListener('pointercancel',cancelGroupDrag);window.addEventListener('blur',cancelGroupDrag)})
onUnmounted(()=>{window.removeEventListener('pointermove',groupPointerMove);window.removeEventListener('pointerup',finishGroupDrag);window.removeEventListener('pointercancel',cancelGroupDrag);window.removeEventListener('blur',cancelGroupDrag)})

const scenes = computed(() => (store.settings.client?.['lumo-launch-scenes'] as LaunchScene[] | undefined) ?? [])
const sceneId = ref(''), sceneName = ref(''), sceneResources = ref<string[]>([]), outcomes = ref<string[]>([])
const invalid = computed(() => store.resources.filter(item=>props.health[item.id]===false))
async function operation(fn:()=>Promise<unknown>) { if(busy.value)return;busy.value=true;try{await fn()}catch(e){store.notify(String(e))}finally{busy.value=false} }
async function moveGroup(id:string,direction:number){await operation(async()=>{const ids=store.groups.map(g=>g.id);const from=ids.indexOf(id),to=from+direction;if(to<0||to>=ids.length)return;[ids[from],ids[to]]=[ids[to]!,ids[from]!];await api.reorderGroups(ids);store.groups=await api.listGroups()})}
async function rename(id:string){await operation(async()=>{const name=names.value[id]?.trim();if(!name)return;await api.renameGroup(id,name);store.groups=await api.listGroups();delete names.value[id]})}
async function removeGroup(id:string,name:string){if(!await askConfirmation(`删除“${name}”分组？启动项会保留并移到未分组。`))return;await operation(async()=>{await api.deleteGroup(id);await store.reload()})}
function editScene(scene?:LaunchScene){sceneId.value=scene?.id??'';sceneName.value=scene?.name??'';sceneResources.value=[...(scene?.resource_ids??[])];outcomes.value=[]}
async function saveScene(){await operation(async()=>{if(!sceneName.value.trim()||!sceneResources.value.length)return;const item={id:sceneId.value||crypto.randomUUID(),name:sceneName.value.trim(),resource_ids:[...sceneResources.value]};await store.saveClient('lumo-launch-scenes',[...scenes.value.filter(s=>s.id!==item.id),item]);editScene()})}
async function removeScene(scene:LaunchScene){if(!await askConfirmation(`删除“${scene.name}”组合？原始启动项会保留。`))return;await operation(()=>store.saveClient('lumo-launch-scenes',scenes.value.filter(s=>s.id!==scene.id)))}
async function launchScene(scene:LaunchScene){await operation(async()=>{outcomes.value=[];for(const id of scene.resource_ids){const item=store.resources.find(r=>r.id===id);try{await store.launchResource(id);outcomes.value.push(`✓ ${item?.name??id} 已发送启动请求`)}catch(e){outcomes.value.push(`✗ ${item?.name??'已移除的启动项'}：${String(e)}`)}}})}
async function removeInvalid(){if(!await askConfirmation(`从应用库移除 ${invalid.value.length} 个失效入口？不会删除原始文件；外接磁盘暂时离线时请先重新连接。`))return;await operation(async()=>{await api.batchResources(invalid.value.map(i=>i.id),'delete',null);await store.reload();emit('refresh')})}
</script>
<template>
  <Dialog v-model:open="opened"><DialogContent class="flex max-h-[85dvh] flex-col sm:max-w-2xl"><DialogHeader><DialogTitle>整理应用库</DialogTitle><DialogDescription>管理分组、检查失效入口，或创建一次打开多个启动项的工作组合。</DialogDescription></DialogHeader>
    <Tabs v-model="tab" class="min-h-0"><TabsList><TabsTrigger value="groups">分组</TabsTrigger><TabsTrigger value="health">失效检查</TabsTrigger><TabsTrigger value="scenes">工作组合</TabsTrigger></TabsList>
      <TabsContent value="groups" class="max-h-[55dvh] overflow-y-auto"><div class="flex flex-col gap-2"><div v-for="(group,index) in store.groups" :key="group.id" :data-group-order="group.id" :class="groupDrop===group.id?'ring-2 ring-primary':draggingGroup===group.id?'opacity-50':''" class="flex flex-wrap items-center gap-2 rounded-lg bg-muted/40 p-2"><Button size="icon-sm" variant="ghost" class="touch-none cursor-grab" :disabled="busy" aria-label="拖动分组排序" @pointerdown="startGroupDrag($event,group.id)"><GripVertical/></Button><Input :model-value="names[group.id]??group.name" :aria-label="`分组 ${group.name}`" class="min-w-32 flex-1" maxlength="80" @update:model-value="value=>names[group.id]=String(value)"/><Button size="icon-sm" variant="ghost" :disabled="busy||index===0" aria-label="上移分组" @click="moveGroup(group.id,-1)"><ArrowUp/></Button><Button size="icon-sm" variant="ghost" :disabled="busy||index===store.groups.length-1" aria-label="下移分组" @click="moveGroup(group.id,1)"><ArrowDown/></Button><Button size="icon-sm" variant="ghost" :disabled="busy||!names[group.id]?.trim()" aria-label="保存分组名称" @click="rename(group.id)"><Pencil/></Button><Button size="icon-sm" variant="danger" :disabled="busy" aria-label="删除分组" @click="removeGroup(group.id,group.name)"><Trash2/></Button></div><p v-if="!store.groups.length" class="py-4 text-sm text-muted-foreground">还没有分组，请在应用库中新建。</p></div></TabsContent>
      <TabsContent value="health" class="max-h-[55dvh] overflow-y-auto"><p class="mb-3 text-sm text-muted-foreground">未连接的移动磁盘也可能显示失效，请先确认磁盘已连接。</p><div class="flex flex-col gap-2"><div v-for="item in invalid" :key="item.id" class="flex items-center gap-2"><span class="min-w-0 flex-1"><span class="block truncate">{{ item.name }}</span><span class="block truncate text-xs text-muted-foreground">{{ item.target }}</span></span><Button variant="outline" size="sm" :disabled="busy" @click="emit('relocate',item)">重新定位</Button></div><p v-if="!invalid.length" class="text-sm text-muted-foreground">没有发现失效入口。</p><div class="mt-3 flex gap-2"><Button variant="outline" :disabled="busy" @click="emit('refresh')">重新检查</Button><Button variant="destructive" :disabled="busy||!invalid.length" @click="removeInvalid">移除失效入口</Button></div></div></TabsContent>
      <TabsContent value="scenes" class="max-h-[55dvh] overflow-y-auto"><div class="flex flex-col gap-3"><div v-for="scene in scenes" :key="scene.id" class="flex flex-wrap items-center gap-2"><span class="min-w-0 flex-1 truncate">{{ scene.name }} · {{ scene.resource_ids.length }} 项</span><Button size="sm" :disabled="busy" @click="launchScene(scene)">打开组合</Button><Button size="sm" variant="outline" :disabled="busy" @click="editScene(scene)">编辑</Button><Button size="icon-sm" variant="danger" :disabled="busy" aria-label="删除组合" @click="removeScene(scene)"><Trash2/></Button></div><FieldGroup><Field><FieldLabel for="scene-name">{{ sceneId?'编辑组合':'新建组合' }}</FieldLabel><Input id="scene-name" v-model="sceneName" maxlength="80" placeholder="例如：开始工作"/></Field><Field><FieldLabel>选择启动项</FieldLabel><div class="max-h-48 overflow-y-auto"><label v-for="item in store.resources" :key="item.id" class="flex items-center gap-2 py-1"><Checkbox :model-value="sceneResources.includes(item.id)" @update:model-value="value=>sceneResources=value===true?[...new Set([...sceneResources,item.id])]:sceneResources.filter(id=>id!==item.id)"/>{{ item.name }}</label></div></Field></FieldGroup><div class="flex gap-2"><Button :disabled="busy||!sceneName.trim()||!sceneResources.length" @click="saveScene">保存组合</Button><Button variant="ghost" :disabled="busy" @click="editScene()">清空</Button></div><p v-for="line in outcomes" :key="line" class="break-words text-sm" role="status">{{ line }}</p></div></TabsContent>
    </Tabs>
  </DialogContent></Dialog>
</template>
