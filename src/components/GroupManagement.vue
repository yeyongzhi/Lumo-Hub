<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { Folder, Plus, Pencil, Search, Trash2, Pin } from '@lucide/vue'
import { api } from '@/lib/api'
import { askConfirmation } from '@/lib/confirm'
import { useWorkspaceStore } from '@/stores/workspace'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Field, FieldLabel } from '@/components/ui/field'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog'

const opened=defineModel<boolean>('open',{default:false})
const store=useWorkspaceStore()
const newGroupName=ref(''),busy=ref(false),query=ref(''),formOpen=ref(false),editingId=ref('')
const filteredGroups=computed(()=>store.groups.filter(group=>group.name.toLocaleLowerCase().includes(query.value.trim().toLocaleLowerCase())))
function openForm(id='',name=''){editingId.value=id;newGroupName.value=name;formOpen.value=true}
async function submitForm(){if(editingId.value){const group=store.groups.find(item=>item.id===editingId.value);if(group){names[group.id]=newGroupName.value;await renameGroup(group.id,group.name)}}else await createGroup()}

async function pinGroup(id:string){busy.value=true;try{await store.saveClient('lumo-launcher-pinned-group',store.settings.client?.['lumo-launcher-pinned-group']===id?null:id);store.notify('快捷启动固定分组已更新')}catch(e){store.notify(String(e))}finally{busy.value=false}}
const names=reactive<Record<string,string>>({})

async function createGroup(){
  const name=newGroupName.value.trim()
  if(!name||busy.value)return
  busy.value=true
  try{await api.createGroup(name);newGroupName.value='';store.groups=await api.listGroups();formOpen.value=false;query.value='';store.notify('分组已创建')}
  catch(error){store.notify(`创建分组失败：${String(error)}`)}
  finally{busy.value=false}
}
async function renameGroup(id:string,currentName:string){
  const name=(names[id]??currentName).trim()
  if(!name||busy.value)return
  if(name===currentName){formOpen.value=false;return}
  busy.value=true
  try{await api.renameGroup(id,name);delete names[id];store.groups=await api.listGroups();formOpen.value=false;store.notify('分组名称已更新')}
  catch(error){store.notify(`重命名分组失败：${String(error)}`)}
  finally{busy.value=false}
}
async function removeGroup(id:string,name:string){
  if(!await askConfirmation(`删除“${name}”分组？其中的启动项会移至未分组。`))return
  busy.value=true
  try{await api.deleteGroup(id);if(store.settings.client?.['lumo-launcher-pinned-group']===id)await store.saveClient('lumo-launcher-pinned-group',null);delete names[id];await store.reload();store.notify('分组已删除')}
  catch(error){store.notify(`删除分组失败：${String(error)}`)}
  finally{busy.value=false}
}
</script>

<template>
  <Dialog v-model:open="opened">
    <DialogContent class="flex h-[min(42rem,85dvh)] flex-col sm:max-w-3xl">
      <DialogHeader><DialogTitle class="flex items-center gap-3">管理分组<Badge variant="secondary">{{ store.groups.length }} 个自定义分组</Badge></DialogTitle><DialogDescription>固定的自定义分组显示在快捷启动的全部应用上方。删除分组不会删除其中的启动项。</DialogDescription></DialogHeader>
      <div class="flex shrink-0 items-center gap-3"><div class="relative min-w-0 flex-1"><Search class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"/><Input v-model="query" class="pl-9" placeholder="搜索分组名称" aria-label="搜索分组"/></div><Button :disabled="busy" @click="openForm()"><Plus data-icon="inline-start"/>新建分组</Button></div>
      <div class="flex shrink-0 items-center justify-between border-b border-border px-3 pb-2 text-xs text-muted-foreground"><span>分组名称</span><span>操作</span></div>
      <ScrollArea class="min-h-0 flex-1">
        <div v-if="filteredGroups.length" class="flex flex-col gap-1 pr-3"><div v-for="group in filteredGroups" :key="group.id" class="flex min-w-0 items-center gap-3 rounded-lg px-3 py-3 hover:bg-muted/50"><Folder class="size-4 shrink-0 text-muted-foreground"/><span class="min-w-0 flex-1 truncate text-sm" :title="group.name">{{ group.name }}</span><Badge v-if="store.settings.client?.['lumo-launcher-pinned-group']===group.id" variant="secondary">已固定</Badge><div class="flex shrink-0 items-center gap-1"><Button size="sm" variant="ghost" :disabled="busy" :aria-label="`编辑分组：${group.name}`" @click="openForm(group.id,group.name)"><Pencil data-icon="inline-start"/>编辑</Button><Button size="sm" variant="danger" :disabled="busy" :aria-label="`删除分组：${group.name}`" @click="removeGroup(group.id,group.name)"><Trash2 data-icon="inline-start"/>删除</Button><Button size="sm" variant="ghost" :disabled="busy" :aria-pressed="store.settings.client?.['lumo-launcher-pinned-group']===group.id" :aria-label="`${store.settings.client?.['lumo-launcher-pinned-group']===group.id?'取消固定':'固定到快捷启动'}：${group.name}`" @click="pinGroup(group.id)"><Pin data-icon="inline-start"/>{{ store.settings.client?.['lumo-launcher-pinned-group']===group.id?'取消固定':'固定' }}</Button></div></div></div>
        <p v-else class="py-16 text-center text-sm text-muted-foreground">{{ query.trim()?'没有匹配的分组':'还没有自定义分组，点击上方新建分组。' }}</p>
      </ScrollArea>
    </DialogContent>
  </Dialog>
  <Dialog v-model:open="formOpen"><DialogContent><DialogHeader><DialogTitle>{{ editingId?'编辑分组':'新建分组' }}</DialogTitle><DialogDescription>输入分组名称，用于整理应用库中的启动项。</DialogDescription></DialogHeader><form class="flex flex-col gap-4" @submit.prevent="submitForm"><Field><FieldLabel for="group-name">分组名称</FieldLabel><Input id="group-name" v-model="newGroupName" maxlength="80" autofocus/></Field><div class="flex justify-end gap-2"><Button type="button" variant="outline" :disabled="busy" @click="formOpen=false">取消</Button><Button type="submit" :disabled="busy||!newGroupName.trim()">{{ busy?'正在保存…':'保存' }}</Button></div></form></DialogContent></Dialog>
</template>
