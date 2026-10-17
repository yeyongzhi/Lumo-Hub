export const launcherSections=[{id:'recent',label:'最近使用'},{id:'pinned',label:'固定项'},{id:'common',label:'系统启动项'},{id:'resource',label:'全部应用'}] as const
export function launcherOrder(client?:Record<string,unknown>){
  const raw=client?.['lumo-launcher-order']
  const ids=launcherSections.map(item=>item.id) as string[]
  return [...new Set([...(Array.isArray(raw)?raw.filter((id):id is string=>typeof id==='string'&&ids.includes(id)):[]),...ids])]
}
export function fixedResourceIds(client?:Record<string,unknown>){
  const raw=client?.['lumo-launcher-fixed-items']
  return Array.isArray(raw)?raw.filter((id):id is string=>typeof id==='string'):[]
}
