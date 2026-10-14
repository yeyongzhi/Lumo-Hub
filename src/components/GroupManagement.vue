<script setup lang="ts">
import { reactive, ref } from 'vue'
import { Folder, Plus, Save, Trash2 } from '@lucide/vue'
import { api } from '@/lib/api'
import { askConfirmation } from '@/lib/confirm'
import { useWorkspaceStore } from '@/stores/workspace'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog'

const opened=defineModel<boolean>('open',{default:false})
const store=useWorkspaceStore()
const newGroupName=ref(''),busy=ref(false)
const names=reactive<Record<string,string>>({})

async function createGroup(){
  const name=newGroupName.value.trim()
  if(!name||busy.value)return
  busy.value=true
  try{await api.createGroup(name);newGroupName.value='';store.groups=await api.listGroups();store.notify('分组已创建')}
  catch(error){store.notify(`创建分组失败：${String(error)}`)}
  finally{busy.value=false}
}
async function renameGroup(id:string,currentName:string){
  const name=(names[id]??currentName).trim()
  if(!name||name===currentName||busy.value)return
  busy.value=true
  try{await api.renameGroup(id,name);delete names[id];store.groups=await api.listGroups();store.notify('分组名称已更新')}
  catch(error){store.notify(`重命名分组失败：${String(error)}`)}
  finally{busy.value=false}
}
async function removeGroup(id:string,name:string){
  if(!await askConfirmation(`删除“${name}”分组？其中的启动项会移至未分组。`))return
  busy.value=true
  try{await api.deleteGroup(id);delete names[id];await store.reload();store.notify('分组已删除')}
  catch(error){store.notify(`删除分组失败：${String(error)}`)}
  finally{busy.value=false}
}
</script>

<template>
  <Dialog v-model:open="opened">
    <DialogContent class="flex max-h-[85dvh] flex-col sm:max-w-xl">
      <DialogHeader><DialogTitle>管理分组</DialogTitle><DialogDescription>创建、重命名或删除应用库中的自定义分组。删除分组不会删除其中的启动项。</DialogDescription></DialogHeader>
      <form class="flex shrink-0 gap-2" @submit.prevent="createGroup"><Input v-model="newGroupName" class="min-w-0" placeholder="输入新分组名称" maxlength="80" aria-label="新分组名称" /><Button type="submit" :disabled="busy||!newGroupName.trim()"><Plus data-icon="inline-start" />新建</Button></form>
      <div class="min-h-0 flex-1 overflow-y-auto"><div v-if="store.groups.length" class="flex flex-col gap-2"><div v-for="group in store.groups" :key="group.id" class="flex min-w-0 items-center gap-2 rounded-lg bg-muted/40 p-2"><Folder class="size-4 shrink-0 text-muted-foreground" /><Input :model-value="names[group.id]??group.name" class="min-w-0 flex-1" maxlength="80" :aria-label="`分组名称：${group.name}`" @update:model-value="value=>names[group.id]=String(value)" /><Button size="icon-sm" variant="outline" :disabled="busy||!((names[group.id]??group.name).trim())||(names[group.id]??group.name)===group.name" :aria-label="`保存分组名称：${group.name}`" title="保存名称" @click="renameGroup(group.id,group.name)"><Save /></Button><Button size="icon-sm" variant="danger" :disabled="busy" :aria-label="`删除分组：${group.name}`" title="删除分组" @click="removeGroup(group.id,group.name)"><Trash2 /></Button></div></div><p v-else class="py-8 text-center text-sm text-muted-foreground">还没有自定义分组。</p></div>
      <p class="text-xs text-muted-foreground">{{ store.groups.length }} 个自定义分组</p>
    </DialogContent>
  </Dialog>
</template>
