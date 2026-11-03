import { api } from '@/lib/api'
import { askConfirmation } from '@/lib/confirm'
import { useWorkspaceStore } from '@/stores/workspace'

export function useGroupOperations() {
  const store=useWorkspaceStore()
  async function create(name:string) {
    await api.createGroup(name.trim())
    store.groups=await api.listGroups()
  }
  async function rename(id:string,name:string) {
    await api.renameGroup(id,name.trim())
    store.groups=await api.listGroups()
  }
  async function reorder(ids:string[]) {
    await api.reorderGroups(ids)
    store.groups=await api.listGroups()
  }
  async function remove(id:string,name:string) {
    if(!await askConfirmation(`删除“${name}”分组？启动项会保留并移到未分组。`))return false
    await api.deleteGroup(id)
    if(store.settings.client?.['lumo-launcher-pinned-group']===id)await store.saveClient('lumo-launcher-pinned-group',null)
    await store.reload()
    return true
  }
  return {create,rename,reorder,remove}
}
