import type { Resource, LaunchOptions } from './types';

export const libraryColumnOptions = [
  { id: 'path', name: '启动路径' },
  { id: 'developer', name: '开发者' },
  { id: 'group', name: '分组' },
  { id: 'recent', name: '最近使用' },
] as const;

export function libraryColumns(client: Record<string, unknown> | undefined): string[] {
  const saved = client?.['lumo-library-columns'];
  if (!Array.isArray(saved)) return ['group'];
  const allowed = new Set<string>(libraryColumnOptions.map((column) => column.id));
  return [
    ...new Set(
      saved.filter((value): value is string => typeof value === 'string' && allowed.has(value)),
    ),
  ];
}

export function libraryIconSize(
  client: Record<string, unknown> | undefined,
): 'small' | 'standard' | 'large' {
  const value = client?.['lumo-library-icon-size'];
  return value === 'small' || value === 'large' ? value : 'standard';
}

function validLibraryFilter(value: unknown, groups: readonly { id: string }[]): value is string {
  if (typeof value !== 'string') return false;
  if (['all', 'app', 'web', 'file', 'favorites', 'common', 'pinned'].includes(value)) return true;
  return value.startsWith('group:') && groups.some((group) => value === `group:${group.id}`);
}

export function libraryDefaultFilter(
  client: Record<string, unknown> | undefined,
  groups: readonly { id: string }[],
): string {
  const value = client?.['lumo-library-default-filter'];
  return value === 'remember' || validLibraryFilter(value, groups) ? value : 'all';
}

export function libraryInitialFilter(
  client: Record<string, unknown> | undefined,
  groups: readonly { id: string }[],
): string {
  const selected = libraryDefaultFilter(client, groups);
  if (selected !== 'remember') return selected;
  const last = client?.['lumo-library-last-filter'];
  return validLibraryFilter(last, groups) ? last : 'all';
}

export function targetKey(kind: string, target: string) {
  if (kind === 'web') {
    try {
      return new URL(target).href;
    } catch {
      return target.trim();
    }
  }
  return target.trim().replace(/\//g, '\\').replace(/\\+$/, '').toLocaleLowerCase();
}
export function launchOptions(
  client: Record<string, unknown> | undefined,
  id: string,
): LaunchOptions {
  const value = client?.[`lumo-resource-options-${id}`] as Partial<LaunchOptions> | undefined;
  return {
    arguments: value?.arguments ?? [],
    working_directory: value?.working_directory ?? '',
    icon_data: value?.icon_data ?? null,
  };
}
export function resourceImage(client: Record<string, unknown> | undefined, item: Resource) {
  return launchOptions(client, item.id).icon_data || item.icon_data;
}

/** Preserve visible order as usage counts change; explicit sort changes rebuild it. */
export function stableResourceOrder(items: Resource[], ids: string[]) {
  const positions = new Map(ids.map((id, i) => [id, i]));
  return [...items].sort(
    (a, b) => (positions.get(a.id) ?? ids.length) - (positions.get(b.id) ?? ids.length),
  );
}
