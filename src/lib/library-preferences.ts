import type { Resource, LaunchOptions } from './types'

export function targetKey(kind: string, target: string) {
  if (kind === 'web') { try { return new URL(target).href } catch { return target.trim() } }
  return target.trim().replace(/\//g, '\\').replace(/\\+$/, '').toLocaleLowerCase()
}
export function launchOptions(client: Record<string, unknown> | undefined, id: string): LaunchOptions {
  const value = client?.[`lumo-resource-options-${id}`] as Partial<LaunchOptions> | undefined
  return { arguments: value?.arguments ?? [], working_directory: value?.working_directory ?? '', icon_data: value?.icon_data ?? null }
}
export function resourceImage(client: Record<string, unknown> | undefined, item: Resource) {
  return launchOptions(client, item.id).icon_data || item.icon_data
}

/** Preserve visible order as usage counts change; explicit sort changes rebuild it. */
export function stableResourceOrder(items: Resource[], ids: string[]) {
  const positions = new Map(ids.map((id, i) => [id, i]))
  return [...items].sort((a, b) => (positions.get(a.id) ?? ids.length) - (positions.get(b.id) ?? ids.length))
}
